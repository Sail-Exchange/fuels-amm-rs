use async_trait::async_trait;
use fuels::{accounts::wallet::Wallet, types::ContractId};
use serde::{Deserialize, Serialize};

use crate::{
    amm::{factory::AutomatedMarketMakerFactory, AMM},
    errors::AMMError,
};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct MiraFactory {
    pub contract_id: ContractId,
    pub creation_block: u64,
    pub fee: u32,
}

#[async_trait]
impl AutomatedMarketMakerFactory for MiraFactory {
    /// Returns the address of the factory.
    fn address(&self) -> ContractId {
        self.contract_id
    }

    /// Gets all Pools from the factory created logs up to the `to_block` block number.
    ///
    /// Returns a vector of AMMs.
    async fn get_all_amms(
        &self,
        to_block: Option<u64>,
        wallet: Wallet,
        step: u64,
    ) -> Result<Vec<AMM>, AMMError> {
        todo!()
    }

    /// Populates all AMMs data via batched static calls.
    async fn populate_amm_data(
        &self,
        amms: &mut [AMM],
        block_number: Option<u64>,
        wallet: Wallet,
    ) -> Result<(), AMMError> {
        todo!()
    }
}
