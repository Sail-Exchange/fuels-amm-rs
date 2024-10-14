pub mod factory;
pub mod oxiswap;
use async_trait::async_trait;
use fuels::{
    accounts::wallet::Wallet,
    types::{AssetId, ContractId},
};
use oxiswap::Oxiswap;
use serde::{Deserialize, Serialize};

use crate::errors::{AMMError, ArithmeticError, SwapSimulationError};

#[async_trait]
pub trait AutomatedMarketMaker {
    /// Returns the address of the AMM.
    fn address(&self) -> ContractId;

    /// Syncs the AMM data on chain via batched static calls.
    async fn sync(&mut self, wallet: Wallet) -> Result<(), AMMError>;

    /// Returns a vector of tokens in the AMM.
    fn tokens(&self) -> Vec<AssetId>;

    /// Calculates a f64 representation of base token price in the AMM.
    fn calculate_price(&self, base_token: AssetId) -> Result<u64, ArithmeticError>;

    /// Populates the AMM data via batched static calls.
    async fn populate_data(
        &mut self,
        block_number: Option<u64>,
        wallet: Wallet,
    ) -> Result<(), AMMError>;

    /// Locally simulates a swap in the AMM.
    ///
    /// Returns the amount received for `amount_in` of `token_in`.
    fn simulate_swap(&self, token_in: AssetId, amount_in: u64) -> Result<u64, SwapSimulationError>;

    /// Locally simulates a swap in the AMM.
    /// Mutates the AMM state to the state of the AMM after swapping.
    /// Returns the amount received for `amount_in` of `token_in`.
    fn simulate_swap_mut(
        &mut self,
        token_in: AssetId,
        amount_in: u64,
    ) -> Result<u64, SwapSimulationError>;

    /// Returns the token out of the AMM for a given `token_in`.
    fn get_token_out(&self, token_in: AssetId) -> AssetId;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(clippy::upper_case_acronyms)]
pub enum AMM {
    Oxiswap(Oxiswap),
}

#[async_trait]
impl AutomatedMarketMaker for AMM {
    fn address(&self) -> ContractId {
        match self {
            AMM::Oxiswap(pool) => pool.address,
        }
    }

    async fn sync(&mut self, wallet: Wallet) -> Result<(), AMMError> {
        match self {
            AMM::Oxiswap(pool) => pool.sync(wallet).await,
        }
    }

    fn tokens(&self) -> Vec<AssetId> {
        match self {
            AMM::Oxiswap(pool) => pool.tokens(),
        }
    }

    fn calculate_price(&self, base_token: AssetId) -> Result<u64, ArithmeticError> {
        match self {
            AMM::Oxiswap(pool) => pool.calculate_price(base_token),
        }
    }

    async fn populate_data(
        &mut self,
        block_number: Option<u64>,
        wallet: Wallet,
    ) -> Result<(), AMMError> {
        match self {
            AMM::Oxiswap(pool) => pool.populate_data(block_number, wallet).await,
        }
    }

    fn simulate_swap(&self, token_in: AssetId, amount_in: u64) -> Result<u64, SwapSimulationError> {
        match self {
            AMM::Oxiswap(pool) => pool.simulate_swap(token_in, amount_in),
        }
    }

    fn simulate_swap_mut(
        &mut self,
        token_in: AssetId,
        amount_in: u64,
    ) -> Result<u64, SwapSimulationError> {
        match self {
            AMM::Oxiswap(pool) => pool.simulate_swap_mut(token_in, amount_in),
        }
    }

    fn get_token_out(&self, token_in: AssetId) -> AssetId {
        match self {
            AMM::Oxiswap(pool) => pool.get_token_out(token_in),
        }
    }
}
