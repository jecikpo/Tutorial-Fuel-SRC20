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

abigen!(
    Contract(
        name = "SRC20",
        abi = "./out/debug/src20-abi.json"
    )
);

pub const DEFAULT_GAS_LIMIT: u64 = 400000;
pub const DEFAULT_SUB_ID: Bits256 = Bits256([0; 32]);

pub const SECRECT_KEY: &str = "<your secret pass goes here>";

pub const FUEL_NETWORK: &str = "127.0.0.1:4000";
//pub const FUEL_NETWORK: &str = "testnet.fuel.network";

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

pub async fn get_src20_contract_instance_with_configurables(configurables: SRC20Configurables) -> (
    SRC20<WalletUnlocked>, 
    ContractId, 
    WalletUnlocked, 
    AssetId) 
{    
    let (provider, wallet, salt) = get_wallet_provider_salt().await;

    let id = Contract::load_from(
        "./out/debug/src20.bin",
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

pub fn create_src20_configurables(name: &str, symbol: &str, decimals: u8) -> SRC20Configurables {
    let name_configurable: SizedAsciiString<5> = name.try_into().unwrap();
    let symbol_configurable: SizedAsciiString<3> = symbol.try_into().unwrap();

    SRC20Configurables::default()
    .with_name(name_configurable).unwrap()
    .with_symbol(symbol_configurable).unwrap()
    .with_decimals(decimals).unwrap()
}

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

