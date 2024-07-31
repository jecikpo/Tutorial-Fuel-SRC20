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

impl ContractInstance<SRC20<WalletUnlocked>> {
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

    pub async fn call_burn(self, sub_id: Bits256, amount: u64) {
        let _ = self.instance.clone()
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
        .await;
    }
}

