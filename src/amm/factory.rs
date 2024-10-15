use async_trait::async_trait;
use fuels::{accounts::wallet::Wallet, types::ContractId};
use serde::{Deserialize, Serialize};

use crate::errors::AMMError;

use super::AMM;
#[async_trait]
pub trait AutomatedMarketMakerFactory {
    /// Returns the address of the factory.
    fn address(&self) -> ContractId;

    /// Gets all Pools from the factory created logs up to the `to_block` block number.
    ///
    /// Returns a vector of AMMs.
    async fn get_all_amms(
        &self,
        to_block: Option<u64>,
        wallet: Wallet,
        step: u64,
    ) -> Result<Vec<AMM>, AMMError>;

    /// Populates all AMMs data via batched static calls.
    async fn populate_amm_data(
        &self,
        amms: &mut [AMM],
        block_number: Option<u64>,
        wallet: Wallet,
    ) -> Result<(), AMMError>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Factory {}
