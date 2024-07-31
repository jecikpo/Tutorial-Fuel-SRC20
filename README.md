# Fuel SRC20 Tutorial
This tutorial explain step by step how to create an SRC20 (Sway Request for Comments 20) 
contract and test it using the Fuel Rust SDK framework. We will focus on the differences 
between FuelVM and EVM which are important to understand in the context of this contract.

Prerequisites:
1) Basic knowledge of Sway and Rust
2) Familiarity with ERC20 standard
3) Fuel SDK installed
4) Knowledge of the UTXO model

This tutorial was written with forc version 0.60.0.

Let's go.

## SRC20 Standard
The SRC20 is the equivalent of EVM ERC20 standard. It's description can be found [here](https://docs.fuel.network/docs/sway-standards/src-20-native-asset/).
Documentation provides information on the API that we are going to implement. There are
couple of differences between the ERC20 and SRC20, primarly because the Fuel supports the 
minted assets natively (they are referred to as Native Assets) and because Fuel uses UTXO
model instead of account model. We will explain those concepts later. Below is the excerpt
from the standard.

The standard requires implementation of the following public functions:

```rust
fn total_assets() -> u64
```
This function MUST return the total number of individual assets for a contract.

```rust
fn total_supply(asset: AssetId) -> Option<u64>
```
This function MUST return the total supply of coins for an asset. This function MUST return 
Some for any assets minted by the contract.

```rust
fn name(asset: AssetId) -> Option<String>
```
This function MUST return the name of the asset, such as “Ether”. This function MUST return 
Some for any assets minted by the contract.

```rust
fn symbol(asset: AssetId) -> Option<String>
```
This function must return the symbol of the asset, such as “ETH”. This function MUST return 
Some for any assets minted by the contract.

```rust
fn decimals(asset: AssetId) -> Option<u8>
```
This function must return the number of decimals the asset uses - e.g. 8, which means 
to divide the coin amount by 100000000 to get its user representation. This function MUST 
return Some for any assets minted by the contract.

You may not two things, there is no `mint()` and `burn()` functions and there are also no 
transfer related functions, like `transfer()` or `transferFrom()`.

The `mint()` and `burn()` functions are defined in a different standard: SRC3. SRC3 requires 
to implement the following public functions:

```rust
fn mint(recipient: Identity, sub_id: SubId, amount: u64)
```
This function MUST mint amount coins with sub-identifier sub_id and transfer them to the 
recipient. This function MAY contain arbitrary conditions for minting, and revert if those 
conditions are not met.

**Mint Arguments**
- `recipient` - The Identity to which the newly minted asset is transferred to.
- `sub_id` - The sub-identifier of the asset to mint.
- `amount` - The quantity of coins to mint.

```rust
fn burn(sub_id: SubId, amount: u64)
```
This function MUST burn amount coins with the sub-identifier sub_id and MUST ensure the 
AssetId of the asset is the sha-256 hash of (ContractId, SubId) for the implementing contract. 
This function MUST ensure at least amount coins have been transferred to the implementing 
contract. This function MUST update the total supply defined in the SRC-20 standard. 
This function MAY contain arbitrary conditions for burning, and revert if those conditions 
are not met.

**Burn Arguments**
- `sub_id` - The sub-identifier of the asset to burn.
- `amount` - The quantity of coins to burn.

The `Identity` and `SubId` will be explained in detail in later sections.

## Native Assets and UTXO
In Ethereum blockchain the ERC20 tokens are managed through the state variable contents of 
the contract. When tokens are minted to an address, the `balanceOf` mapping is updated to 
reflect the balance of tokens of an account. This is not the case in Fuel. Here when an 
asset is created (in Fuel we don't have Tokens, we have Coins, however in this tutorial
I will use the terms Tokens and Coins interchanbly) an UTXO is created with the desired
amount of Coins and it is transferred to an owner's address. You can read more about 
the concept of UTXO [here](https://en.wikipedia.org/wiki/Unspent_transaction_output) and [here](https://learnmeabitcoin.com/technical/transaction/utxo/).

A UTXO created by our SRC20 contract containing some Coins can be treated as Fuel's Native 
Asset. This means that it can be handled similarly to Ether on Ethereum. It can be transfered
to some account independently of the SRC20 smart contract or it can be sent along with 
a call to any contract.

This has some interesting implications. For example the SRC20 contract doesn't have 
any transfer functions. All Coin transfers are handled through Fuel natively hence any specific
actions on token transfers cannot be implemented.

We also don't have approvals and approved accounts on Fuel.

## Identifiers

*Contract ID* uniquely identifies a deployed contract and is created as a result of transaction
of type Create. The number is generated in the following way:

```
sha256(0x4655454C ++ tx.data.salt ++ root(tx.data.witnesses[bytecodeWitnessIndex].data) ++ root_smt(tx.storageSlots))
```

More information can be found [here](https://docs.fuel.network/docs/specs/identifiers/contract-id/)

The Contract ID will be used to send Coins to a contract.

*Asset ID* uniquely identifies a Fuel Native Asset. Once we start minting Coins they will have 
an Asset ID which will be inside the generated UTXOs. The Asset ID is generated in the following way:

```
sha256(CONTRACT_ID ++ SUB_IDENTIFIER)
```

Where the `SUB_IDENTIFIER` (later referred as Sub ID) is an identifier that uniquely identifies 
an Asset within the contract. In our example here we will create an SRC20 contract with just 
a single asset and we will use the default Sub ID which has a value of zero.

The Asset Id of transfered Coins to a contract can be verified.

*Address* is an EOA address.

All three identifiers (Contract ID, Asset ID and Address) are of 256 bits length.

*Identity* - This is not a specific network identifier *per se*, but rather a commonly used data 
structure in Sway that represents either Contract ID or an Address. it is defined in Sway 
in the following way:

```rust
pub enum Identity {
    Address: Address,
    ContractId: ContractId,
}
```

We will use it in our tests.

## Contract Implementation

Let's start by creating our Sway project:

```bash
forc init SRC20
```
The above command will create the `SRC20` directory of the project and it's basic folder structure.
The contract code will be in `src/main.sw` file and `Forc.toml` will contain forc settings. 
Optionally you can change the name on of the `src/main.sw` file to something more meaningful, e.g. 
`src/src20.sw` (this will be helpfull later, if you have multiple contracts as part of the same
project). If you decide to change the name of the Sway source file you should also change the 
compiler entry point in the `Forc.toml` to reflect the new name: `entry = "src20.sw"`.

Now, we can start writing our contract in `src/src20`. Firstly we need to define the ABI (Application
Binary Interface) of our contract, however as we are following the defined standards of SRC20 and SRC3
we can import them. Let's add the following line into the contract (just after the `contract;` 
statement):

```rust
use standards::{
    src20::SRC20,
    src3::SRC3,
};

use std::{
    asset::{
        burn,
        mint_to,
    },
    call_frames::msg_asset_id,
    context::msg_amount,
    constants::DEFAULT_SUB_ID,
    string::String, 
    storage::storage_api::{
        read, 
        write
    },
    asset_id::*,
};
```

We should also add the following to our `Forc.toml` to successfully import the Sway standards:

```conf
[dependencies]
standards = { git = "https://github.com/FuelLabs/sway-standards", tag = "v0.5.1" }
```

It is generally a good practice to import ABI standards of the SRCs. Even if we are developing 
our own unique ABI it is best to define them in a library file and import them through the `use`
keyword.

Now let's remove the existing `abi` and `impl` statements that were created by `forc` and add `impl`
statements for SRC20 and SRC3, you entire Sway `src/src20.sw` file should look now like this:

```rust
contract;

use standards::{
    src20::SRC20,
    src3::SRC3,
};

use std::{
    asset::{
        burn,
        mint_to,
    },
    call_frames::msg_asset_id,
    context::msg_amount,
    constants::DEFAULT_SUB_ID,
    string::String, 
    storage::storage_api::{
        read, 
        write
    },
    asset_id::*,
};

impl SRC20 for Contract {

}

impl SRC3 for Contract {

}
```

Have a look at [SRC20](https://github.com/FuelLabs/sway-standards/blob/master/standards/src/src20.sw) and [SRC3](https://github.com/FuelLabs/sway-standards/blob/master/standards/src/src3.sw) imported library files.
They work similary to traits in Rust.

You probably noticed that there are some attributes before each function. The `storage` attribute is
needed when our function either reads or writes to any storage slot. The `payable` attribute is 
required if it needs to accept UTXO Coin Input. It is required by the `burn()` function, because 
to burn Coins they need to be first transferred to the contract. 

Now we should add the only storage variable of this contract:

```rust
storage {
    total_supply: u64 = 0,
}
```
It will hold the total amount of Coins minted.

Let's also add some configurable parameters of the contract. They can be set inside contract deployment
transaction, once deployed they are immutable. This is similar to setting immutable variables in Solidity inside 
a constructor. Note that Sway contracts don't have a constructor and the configurable variables 
are not held in storage.

```rust
configurable {
    name: str[5] = __to_str_array("Token"),
    symbol: str[3] = __to_str_array("TKN"),
    decimals: u8 = 9,
}
```
We have here three parameters which should be self descriptive in case you are familiar with ERC20. 
They are also set here to default values, hence if they are not specified during contract deployment
the values above shall be assigned.

Now we can start coding our methods. Let's start with the simplest one. All methods that are 
part of the ABI should land within the `impl SRC20 for Contract` or `impl SRC3 for Contract` clauses, 
generally this should match the `abi` defintions. The first one will be:

```rust
    #[storage(read)]
    fn total_assets() -> u64 {
        1
    }
```

The `total_assets()` method returns the total amount of individual assets that are minted by this contract.
We have only one asset here (one AssetId) hence we can safely hardcode the returned value of 1. This method
shall have a more complex logic if number of assets is dynamic (e.g. in case of an NFT). Note that the storage
attribute is not necessary here because we are not touching any of the storage variables, yet we need to include
it because our imported `abi` defines this function with it.

Next we create the `total_supply()`:

```rust
    #[storage(read)]
    fn total_supply(asset: AssetId) -> Option<u64> {
        if asset == AssetId::default() {
            Some(storage.total_supply.read())
        } else {
            None
        }
    }
```

This function takes as an argument `AssetId`, to determine which Asset Id you want to get the total supply of.
In our case we have only a single `AssetId` which is the default one (The default Asset Id is created with 
Sub Id zero). Note here the following:
1) we are reading the `total_supply` value from storage and hence we need the `storage(read)` attribute.
2) The return value is `Option` in case a non-default `AssetId` is provided the function returns `None`. This
is required by the standard.

Next we implement `name()`, `symbol()` and `decimals()` which are similar to the previous function:

```rust
    #[storage(read)]
    fn name(asset: AssetId) -> Option<String> {
        if asset == AssetId::default() {
            Some(String::from_ascii_str(from_str_array(name)))
        } else {
            None
        }
    }

    #[storage(read)]
    fn symbol(asset: AssetId) -> Option<String> {
        if asset == AssetId::default() {
            Some(String::from_ascii_str(from_str_array(symbol)))
        } else {
            None
        }
    }

    #[storage(read)]
    fn decimals(asset: AssetId) -> Option<u8> {
        if asset == AssetId::default() {
            Some(decimals)
        } else {
            None
        }
    }
```
The inner mechanics of those three functions is similar to `total_supply()`, note here that the SRC20 standard
does not impose limitations on the length of the strings returned.

Now we can switch to implementation of the SRC3 standard (minting and burning functions). Yes, they will be 
residing in the `impl SRC3 for Contract` clause, we will start with `mint()`:

```rust
    #[storage(read, write)]
    fn mint(recipient: Identity, sub_id: SubId, amount: u64) {
        require(sub_id == DEFAULT_SUB_ID, "Incorrect Sub Id");
 
        // Increment total supply of the asset and mint to the recipient.
        storage.total_supply.write(amount + storage.total_supply.read());
        mint_to(recipient, DEFAULT_SUB_ID, amount);
    }
```
Let's break it down. First we check if the Sub Id provided is the default one. Remember that we are minting 
only a single asset here and this is supposed to be the default one. Second we update the storage `total_supply`
value by incrementing it by `amount`. The last step is minting the Coins and transferring them to the recipient.

`mint_to()` is in fact doing two things. First it is minting the coins, and then transferring them to a given
`Identity`. There is another function in the standard library - `mint()` and it only mints the Coins, they 
stay on the contract.

The last function that we need to add is `burn()`

```rust
    #[payable]
    #[storage(read, write)]
    fn burn(sub_id: SubId, amount: u64) {
        require(sub_id == DEFAULT_SUB_ID, "Incorrect Sub Id");
        require(msg_amount() == amount, "Incorrect amount provided");
        require(
            msg_asset_id() == AssetId::default(),
            "Incorrect asset provided",
        );
 
        // Decrement total supply of the asset and burn.
        storage.total_supply.write(storage.total_supply.read() - amount);
        burn(DEFAULT_SUB_ID, amount);
    }
```

Let's see what's inside. The first `require` statement checks if the Sub Id we provide is the 
default one. We don't really need to check that as the only asset we can burn will be the default one, but at least 
we want to use this to provide the revert reason to the user. Next `require` checks if the provided amount in
the Input equal to the requested `amount`. We use the `==` operator here to make sure that the user doesn't 
send more Coins that he wishes to burn, in such case the remaining amount would get stuck on the contract.
Finally the third `require` statement checks if the sent asset is the Native Asset handled by that contract.
This is an interesting thing because it is unique to Fuel. Do you remember that all Coins on Fuel are treated
as Native Assets? (Just like Ether on Ethereum) This check validates that the user doesn't send some other 
asset to the contract when burning.

Now we have our contract complete. Make sure that it looks the same as [the one here](https://github.com/jecikpo/Tutorial-Fuel-SRC20/blob/main/SRC20/src/src20.sw)

Try to compile it now, it will issue couple of warnings related to the standards, but you can ignore them 
for now. Issue the following command from the project main directory:
```bash
forc build
```

Once we have our contract compiled succesfully we can see that couple of files were generated:
- `SRC20/out/debug/src20-abi.json` - contains the JSON description of the ABI (we will reference this file
in our tests later).
- `SRC20/out/debug/src20-storage_slots.json` - is a JSON description of the storage slots used be the contract.
We won't need that file for now, but it won't hurt to have a look.
- `SRC20/out/debug/src20.bin` - the smart contract bytecode.

## Testing
This section describes the testing framework I came up with to test my contracts on Fuel using the Fuel's 
Rust SDK. My framework is a bit opinionated and contains a bit of overhead, but I decided to make it this way 
so that writing tests resembles the case of how it is done in Foundry for EVM.

We will do the testing either on the local Fuel node, or the testnet.

### Testing Framework Overview
Let's first describe the testing framework I created for test smart contracts for Fuel. I'm dividing my 
framework into the following files:
- `tests/harness.rs` - the main test file contains just `mod` statements to include other files.
- `tests/utils` directory - all helpers, wrappers and setup functions reside here.
- `tests/utils/setup.rs` - file contains helper functions for setting up wallets and creation of smart contract
instances.
- `tests/utils/instance.rs` - generic trait and implemenation of a smart contract instance.
- `tests/utils/<contract>.rs` - in this case it will be `tests/utils/src20.rs`, a dir containing wrapper functions
of a specific smart contracts callable methods (all enclosed in an `impl`).
- `tests/<contract>/` - directory containing the actual test functions. There could be multiple files here. 
I separate the tests by the functionality being tested, between different files. In this case we will have 
only one file here `tests/src20/coin.rs`.

### Testing Framework Implementation
Let's start by running the following command to initiate the Cargo testing framework in our project directory:

```bash
cargo generate --init fuellabs/sway templates/sway-test-rs --name SRC20 --force
```

After running the command you should have the `tests/harness.rs` file created. Now go to it and delete all 
it's contents, we don't need them right now. Let's put only the following lines into it:

```rust
mod utils;
mod src20;
```

Now we shall create all the required test files:
```bash
mkdir tests/utils
mkdir tests/src20
touch tests/utils/mod.rs
touch tests/utils/setup.rs
touch tests/utils/instance.rs
touch tests/utils/src20.rs
touch tests/src20/mod.rs
touch tests/src20/coin.rs
```

And let's start filling them in. First the `tests/utils/mod.rs` should contain the following:
```rust
pub mod setup;
pub mod src20;
pub mod instance;
```

`tests/src20/mod.rs` the following:
```rust
pub mod coin;
```

The interesting part starts to happen here `tests/utils/setup.rs`. We need to start with some Fuel
specific imports:
```rust
use fuels::{
    prelude::*, 
    types::ContractId, 
    crypto::SecretKey, 
    types::{
        AssetId,
        Bytes32,
        Bits256,
        SizedAsciiString,
    }
};

use rand::Rng;
use std::str::FromStr;
use sha2::{Digest, Sha256};
```

We also need to add the following in order to use `rand` and `sha2` libs to the `Cargo.toml` 
file under our project directory:

```conf
[dependencies]
rand = "0.8"
sha2 = { version = "0.10.7" }
```

Then we need to add the `abigen` macro with our contract's name and a path to the :
```rust
abigen!(
    Contract(
        name = "SRC20",
        abi = "./SRC20/out/debug/src20-abi.json"
    )
);
```
This macro will initialize certain types related to our smart contract and it's specific ABI. To 
see details of what the macro generates you can have a look [here](https://docs.fuel.network/docs/fuels-rs/abigen/).
We need to provide path to the JSON abi file generated by `forc`.

We need to also create some constants:
```rust
pub const DEFAULT_GAS_LIMIT: u64 = 400000;
pub const DEFAULT_SUB_ID: Bits256 = Bits256([0; 32]);

pub const SECRECT_KEY: &str = "<your secret key here>";

pub const FUEL_NETWORK: &str = "127.0.0.1:4000";
//pub const FUEL_NETWORK: &str = "testnet.fuel.network";
```
The most important here is the `SECRET_KEY`, this will be used to sign all your transactions. To obtain
your account secret key (assuming that you have your wallet created already) type the following:
```bash
forc wallet account 0 private-key
```
`FUEL_NETWORK` defines the network that we connect to, for the current tutorial I will assume that a 
local node is used. Once we have the test ready, we will show here how to launch it.

Our first function will be one that creates a `Provider`, a `WalletUnlocked` and some `salt`:
```rust
pub async fn get_wallet_provider_salt() -> (Provider, WalletUnlocked, Salt) {
    // Launch a local network and deploy the contract
    let provider = Provider::connect(FUEL_NETWORK).await.unwrap();

    let secret = match SecretKey::from_str(
        SECRECT_KEY
    ) {
        Ok(value) => value,
        Err(e) => panic!("unable to create secret: {}", e),
    };

    let wallet = WalletUnlocked::new_from_private_key(secret, Some(provider.clone()));

    // Generate a random 32-byte array
    let mut rng = rand::thread_rng();
    let mut bytes = [0u8; 32];
    rng.fill(&mut bytes);

    let salt = Salt::new(bytes);

    (provider, wallet, salt)
}
```
In this function we create the `provider` object which represents a connection to a live node, a `wallet`
which we will use for signing our transaction, our Identity and the `salt`. The `salt` is needed 
to create unique Contract Id. As of writing this, the Rust SDK uses default salt value of zero and hence
each next deployment of the same bytcode will result in a failure. Refer to [this page](https://docs.fuel.network/docs/specs/identifiers/contract-id/) on how 
the Contract Id value is computed.

The above `get_wallet_provider_salt()` function is contract independent and can be reused with different
projects or contracts.

Now we will add a function to create an instance of our SRC20 contract:
```rust
pub async fn get_src20_contract_instance() -> (SRC20<WalletUnlocked>, ContractId, WalletUnlocked, AssetId) {
    
    let (provider, wallet, salt) = get_wallet_provider_salt().await;

    let id = Contract::load_from(
        "./out/debug/src20.bin",
        LoadConfiguration::default().with_salt(salt),
    )
    .unwrap()
    .deploy(&wallet, TxPolicies::default()
        .with_script_gas_limit(DEFAULT_GAS_LIMIT)
        .with_max_fee(DEFAULT_GAS_LIMIT)
    )
    .await
    .unwrap();

    let instance = SRC20::new(id.clone(), wallet.clone());
    let base_asset_id = provider.base_asset_id();

    (instance, id.into(), wallet, *base_asset_id)
}
```
It loads the contract's bytecode from the specified file and deploys it. Returns the following:
- `instance` - instance of the contract. This object will be used to call our ABI methods.
- `id.into()` - is the Contract Id
- `wallet` - we will need it later in our tests to use the deployer's identity
- `*base_asset_id` - is the ID of the base asset.

A word of explanation regarding the Base Asset. Base Asset on FuelVM is Ether. Only Base Asset can be used to pay 
gas fees. Don't confuse it with Native Asset which refers to all assets created on Fuel that are handled using UTXOs.

We will now create one additional function, very similar to the above one, but this one will take the 
`SRC20Configurables` type argument. This will allow us to set values of our configurable variables during
contract deployment. 

```rust
pub async fn get_src20_contract_instance_with_configurables(configurables: SRC20Configurables) -> (
    SRC20<WalletUnlocked>, 
    ContractId, 
    WalletUnlocked, 
    AssetId) 
{    
    let (provider, wallet, salt) = get_wallet_provider_salt().await;

    let id = Contract::load_from(
        "./SRC20/out/debug/src20.bin",
        LoadConfiguration::default()
        .with_salt(salt)
        .with_configurables(configurables),
    )
    .unwrap()
    .deploy(&wallet, TxPolicies::default().with_script_gas_limit(400000).with_max_fee(400000))
    .await
    .unwrap();

    let instance = SRC20::new(id.clone(), wallet.clone());
    let base_asset_id = provider.base_asset_id();

    (instance, id.into(), wallet, *base_asset_id)
}
```

You might have noticed that we didn't explicitly define `SRC20Configurables` and `SRC20` types. This is true,
those type are created by the `abigen` macro.

Next we will create a function that generates the `SRC20Configurables` object containing our values:
```rust
pub fn create_src20_configurables(name: &str, symbol: &str, decimals: u8) -> SRC20Configurables {
    let name_configurable: SizedAsciiString<5> = name.try_into().unwrap();
    let symbol_configurable: SizedAsciiString<3> = symbol.try_into().unwrap();

    SRC20Configurables::default()
    .with_name(name_configurable).unwrap()
    .with_symbol(symbol_configurable).unwrap()
    .with_decimals(decimals).unwrap()
}
```

We also need some helper functions, to be able to calculate correct value of a default Asset Id
for a given Contract Id:
```rust
pub fn get_asset_id(sub_id: Bytes32, contract: ContractId) -> AssetId {
    let mut hasher = Sha256::new();
    hasher.update(*contract);
    hasher.update(*sub_id);
    AssetId::new(*Bytes32::from(<[u8; 32]>::from(hasher.finalize())))
}

pub fn get_default_asset_id(contract: ContractId) -> AssetId {
    let default_sub_id = Bytes32::from([0u8; 32]);
    get_asset_id(default_sub_id, contract)
}
```

Let's shift to the `tests/utils/instance.rs` file. It will contain certain Generic logic for our 
smart contract testing implementation. This code is a bit of overhead, because it is not needed if you 
are working on a single smart contract dapp. However once you have more than one contract deployed
using your test environment this will avoid duplication of certain code.

You can just copy the entire file from [Tutorial repo](https://github.com/jecikpo/Tutorial-Fuel-SRC20/blob/main/tests/utils/instance.rs).

Now we will focus on the wrappers for calling the ABI methods. As you will see the method chains required
to call a smart contract are quite long hence wrapper usage is absolutely necessary if we want to keep the 
tests readable.

We will be writing all that in the `tests/utils/src20.rs` file. First let's add some imports:
```rust
use fuels::{
    prelude::*, 
    types::{
        AssetId,
        Bits256,
        Identity,
    }
};

use crate::utils::setup::{
    SRC20,
    SRC20Configurables,
    get_src20_contract_instance,
    get_src20_contract_instance_with_configurables,
    get_default_asset_id,
    DEFAULT_GAS_LIMIT,
};

use crate::utils::instance::{
    ContractInstance,
};
```
As you can see we will be using the foundations that we created in the `setup.rs` file. 

Now we will create an `impl` block which will define our contructor and methods on the `ContractInstance` 
(defined at `instance.rs`) of type `SRC20<WalletUnslocked>` (defined by `abigen`). This may seem a bit 
complicated in the beginning, but you will see that the resulting outcome of writing the tests in a simple 
manner is worth it. We will put all our remaining code within the below `impl`:

```rust
impl ContractInstance<SRC20<WalletUnlocked>> {

}
```

let's start with constructors. We will have two of them. One with configurables and one without:
```rust
    pub async fn new() -> Self {
        let (instance, contract_id, wallet, _base_asset_id) = get_src20_contract_instance().await;
        Self {
            instance,
            contract_id,
            wallet,
            gas_limit: DEFAULT_GAS_LIMIT,
        }
    }

    pub async fn new_with_configurables(configurables: SRC20Configurables) -> Self {
        let (instance, contract_id, wallet, _base_asset_id) = get_src20_contract_instance_with_configurables(configurables).await;
        Self {
            instance,
            contract_id,
            wallet,
            gas_limit: DEFAULT_GAS_LIMIT,
        }
    }
```

Then go the wrapper methods. My naming convention is to start those method's name with a `call_` prefix:

```rust
    pub async fn call_name(self, asset_id: AssetId) -> Option<String> {
        self.instance.clone()
            .with_account(self.wallet.clone())
            .methods()
            .name(asset_id) // <- smart contract method
            .with_tx_policies(
                TxPolicies::default()
                .with_script_gas_limit(self.gas_limit)
            )
            .call()
            .await
            .unwrap()
            .value
    }
```
let's break it down:
- `instance` - represents a deployed instance of our contract created by the constructor.
- `with_account()` - specifies the wallet/account that we use for calling this contract. That's why we
keep the `wallet` as part of the `ContractInstance` struct.
- `methods()` - is getting us the contract's methods.
- `name()` - is the actual contract method from the ABI that we are calling in this wrapper.
- `with_tx_policies()` - we are using it to specify the gas limit, as the SDK's defaults are as of writing
this incorrect and will lead to a failed call.
- `call()` - performs the contract call.
- `await` - removes the `Future` as this is an `async` call.
- `unwrap()` - unwraps the call `Result`
- `value` - is the returned value by the contract. It should be of type `Option<String>`.

Let's add the similar wrappers: `call_symbol()`, `call_total_supply()` and `call_decimals()`, they shouldn't need further explanation
as they don't introduce anything new:

```rust
    pub async fn call_symbol(self, asset_id: AssetId) -> Option<String> {
        self.instance.clone()
        .with_account(self.wallet.clone())
        .methods()
        .symbol(asset_id) // smart contract function
        .with_tx_policies(
            TxPolicies::default()
            .with_script_gas_limit(self.gas_limit)
        )
        .call()
        .await
        .unwrap()
        .value
    }

    pub async fn call_decimals(self, asset_id: AssetId) -> Option<u8> {
        self.instance.clone()
        .with_account(self.wallet.clone())
        .methods()
        .decimals(asset_id) // smart contract function
        .with_tx_policies(
            TxPolicies::default()
            .with_script_gas_limit(self.gas_limit)
        )
        .call()
        .await
        .unwrap()
        .value
    }

    pub async fn call_total_supply(self, asset_id: AssetId) -> Option<u64> {
        self.instance.clone()
            .with_account(self.wallet)
            .methods()
            .total_supply(asset_id)
            .with_tx_policies(
                TxPolicies::default()
                .with_script_gas_limit(self.gas_limit)
            )
            .call()
            .await
            .unwrap()
            .value
    }
```

We are left with two more wrappers: `mint()` and `burn()`. Let's start with `mint()`:
```rust
    pub async fn call_mint(self, recipient: Identity, sub_id: Bits256, amount: u64) {
        self.instance.clone()
        .with_account(self.wallet)
        .methods()
        .mint(recipient, sub_id, amount)
        .append_variable_outputs(1)
        .with_tx_policies(
            TxPolicies::default()
            .with_script_gas_limit(self.gas_limit)
        )
        .call()
        .await
        .unwrap();
    }
```

The thing that probably cought your eye is the `append_variable_output(1)` extra method used here.
This is required if our contract might transfer some assets to a specific address. The transaction
created for calling such a contract's method need to specify the appropriate number of Variable Outputs.
If those outputs are not specified and the call transfers assets, the transaction will fail. As we 
are going to mint some Coins here and transfer them to the `recipient` we are adding one Variable
Output.

Our last wrapper method is the `burn()`. It should look like this:
```rust
    pub async fn call_burn(self, sub_id: Bits256, amount: u64) {
        self.instance.clone()
        .with_account(self.wallet)
        .methods()
        .burn(sub_id, amount)
        .with_tx_policies(
            TxPolicies::default()
            .with_script_gas_limit(self.gas_limit)
        )
        .call_params(CallParameters::new(
            amount,
            get_default_asset_id(self.contract_id),
            self.gas_limit,
        )).unwrap()
        .call()
        .await
    }
```
The thing that we are doing here differently is the `call_params()` method presence. We use it to 
transfer the `amount` of our contract's deafult Asset Id as part of the call so they can be burned.

Now you should have your entire testing framework complete and the only work that is left is to write 
the tests. Before you head on to the next section make sure that your files are complete by comparing
them with those in the [Tutorial repo](https://github.com/jecikpo/Tutorial-Fuel-SRC20/blob/main/tests/). With the exception of the `tests/src20/coin.rs` which we have yet to fill in.

### SRC20 Tests

Now it's time to finally write our tests into the `tests/src20/coin.rs` file. First we need to 
add imports:
```rust
use crate::utils::setup::*;
use crate::utils::instance::*;

use fuels::{
    prelude::*,
};
```

Each test function's name must:
- start with a `test_` prefix.
- have `#[tokio::test]` attribute.

Let's start with a basic test where we call the `name()` method:

```rust
#[tokio::test]
async fn test_src20_name() {
    let coin = ContractInstance::<SRC20<WalletUnlocked>>::new().await;
    let mut result = coin.clone().call_name(
        get_default_asset_id(coin.contract_id())
    ).await;
    assert_eq!(
        result, 
        Some(String::from("Token"))
    );
}
```

Let's explain it. First we call the `new()` constructor to instantiate the contract. Then we use 
our wrapper method `call_name()` to call the contract's `name()` method. 
Next we compare the `result` with the expect value. Not here that we didn't use the constructor
with configurables, so we are expecting the default value. Note here that we are using the
helper function `get_default_asset_id()` from `setup.rs`, to create the correct default Asset Id
from the Contract Id.

Now you should be able to run the test from the main project directory:
```bash
cargo test
```

And you should see something like this:

```
[...]
test src20::coin::test_src20_name ... ok
[...]
```
Which means that your test passed. Now as an exercise you can write similar tests for:
- `symbol()`
- `decimals()`
- `total_supply()`

Now re-run Cargo and see if your tests were passed.

Once you got here, let's write a test to see if our configurables work. We will need to user our 
second constructor, here is the entire test:
```rust
#[tokio::test]
async fn test_src20_configurables() {
    let name = "SRC20";
    let symbol = "S20";
    let decimals = 10;
    let configurables = create_src20_configurables(name, symbol, decimals);
    let token = ContractInstance::<SRC20<WalletUnlocked>>::new_with_configurables(configurables).await;

    let result_symbol = token.clone().call_symbol(
        get_default_asset_id(token.contract_id())
    ).await;
    assert_eq!(
        result_symbol, 
        Some(String::from(symbol))
    );

    let result_name = token.clone().call_name(
        get_default_asset_id(token.contract_id())
    ).await;

    assert_eq!(
        result_name, 
        Some(String::from(name))
    );

    let result_decimals = token.clone().call_decimals(
        get_default_asset_id(token.contract_id())
    ).await;

    assert_eq!(
        result_decimals, 
        Some(decimals as u8)
    );
}
```

We created `configurables` with our `create_src20_configurables()` helper function and in this test
we call all three methods and compare the results.

Now let's test minting. We will create an empty body of our test function:

```rust
#[tokio::test]
async fn test_src20_mint() {

}
```

Now we can create the contract instance and define the amount that we want to mint:

```rust
    let token = ContractInstance::<SRC20<WalletUnlocked>>::new().await;
    let amount = 1000;
```

Then we record the current balance of the Coin on the deployer account:
```rust
    let balance_before = token.clone()
        .deployer_balance(
            get_default_asset_id(
                token.contract_id()
            )
        ).await;
```
We will use it later on for comparison after minting. Now we mint the asset:
```rust
    token.clone().call_mint(
        token.clone().deployer_identity(),
        DEFAULT_SUB_ID,
        amount
    ).await;
```
And record the balance after minting:
```rust
    let balance_after = token.clone()
        .deployer_balance(
            get_default_asset_id(
                token.contract_id()
            )
        ).await;
```
The last thing, is that should compare the account balance before minting with the one after. The 
Balance before plus the minted amount should be equal to the balance after:
```rust
    assert_eq!(
        balance_before + amount, 
        balance_after
    )
```

And that concludes the minting test. You can re-run Cargo and see if the test is passed. So far your 
output should look like this:

```
[...]
test src20::coin::test_src20_configurables ... ok
test src20::coin::test_src20_mint ... ok
test src20::coin::test_src20_name ... ok
test src20::coin::test_src20_symbol ... ok
test src20::coin::test_src20_total_supply ... ok
[...]
```

Our last test will involve the `burn()`. First we need to mint the Coins, record the balance, burn the 
Coins and record balance again. Balance before should be equal to the balance after plus the burned amount.
Try writing the test yourself. The test code can be found [here](https://github.com/jecikpo/Tutorial-Fuel-SRC20/blob/main/tests/src20/coin.rs)

## Summary
Now you should be able to write a basic Sway contract and test it with Fuel's Rust SDK. Of course the contract
here is basic and not really useful, but for the purpose of this demonstration should suffice.

It is important to remember key differences that we learned here about Fuel vs. Ethereum:
1) The usage of UTXO instead of account model has some interesting implications on how the asset mechanics 
work. We learned to some things with UTXO model's are not possible e.g. actions on Coin transfers.
2) Native Asset handling. Assets minted in FuelVM can sent directly along the contract calls just like
Ether in Ethereum. This also has some interesting implications as there is approvals and no `transferFrom`.
3) The capability of sending any Native Asset along with a call leads to the need of having an additional
verification step - what asset is actually being receved, along with the amount. This could lead in the future
to some security implications.
