use async_trait::async_trait;
use fuels::{
    accounts::wallet::Wallet,
    types::{AssetId, ContractId},
};
use serde::{Deserialize, Serialize};

use super::AutomatedMarketMaker;
use crate::errors::{AMMError, ArithmeticError, SwapSimulationError};

/// Represents an Oxiswap pool.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct Oxiswap {
    pub address: ContractId,
    pub token_a: AssetId,
    pub token_b: AssetId,
    pub reserve_a: u64,
    pub reserve_b: u64,
    pub fee: u64,
}

#[async_trait]
impl AutomatedMarketMaker for Oxiswap {
    fn address(&self) -> ContractId {
        self.address
    }

    /// Synchronizes the AMM's state with the blockchain.
    async fn sync(&mut self, wallet: Wallet) -> Result<(), AMMError> {
        let (reserve_a, reserve_b) = self.get_reserves(wallet).await?;
        self.reserve_a = reserve_a;
        self.reserve_b = reserve_b;
        Ok(())
    }

    fn tokens(&self) -> Vec<AssetId> {
        vec![self.token_a, self.token_b]
    }

    /// Calculates the price of the base token in terms of the other token.
    fn calculate_price(&self, base_token: AssetId) -> Result<u64, ArithmeticError> {
        let (reserve_in, reserve_out) = if base_token == self.token_a {
            (self.reserve_a, self.reserve_b)
        } else {
            (self.reserve_b, self.reserve_a)
        };

        reserve_out
            .checked_div(reserve_in)
            .ok_or(ArithmeticError::DivisionByZero())
    }

    /// Populates the AMM's data from the blockchain.
    async fn populate_data(
        &mut self,
        _block_number: Option<u64>,
        wallet: Wallet,
    ) -> Result<(), AMMError> {
        *self = self.get_pool_info(wallet).await?;
        Ok(())
    }

    /// Simulates a swap without modifying the AMM's state.
    fn simulate_swap(&self, token_in: AssetId, amount_in: u64) -> Result<u64, SwapSimulationError> {
        let (reserve_in, reserve_out) = if token_in == self.token_a {
            (self.reserve_a, self.reserve_b)
        } else {
            (self.reserve_b, self.reserve_a)
        };

        self.get_amount_out(amount_in, reserve_in, reserve_out)
    }

    /// Simulates a swap and updates the AMM's state.
    fn simulate_swap_mut(
        &mut self,
        token_in: AssetId,
        amount_in: u64,
    ) -> Result<u64, SwapSimulationError> {
        let (amount_out, new_reserve_in, new_reserve_out) = if self.token_a == token_in {
            let amount_out = self.get_amount_out(amount_in, self.reserve_a, self.reserve_b)?;
            (
                amount_out,
                self.reserve_a + amount_in,
                self.reserve_b - amount_out,
            )
        } else {
            let amount_out = self.get_amount_out(amount_in, self.reserve_b, self.reserve_a)?;
            (
                amount_out,
                self.reserve_b + amount_in,
                self.reserve_a - amount_out,
            )
        };

        if self.token_a == token_in {
            self.reserve_a = new_reserve_in;
            self.reserve_b = new_reserve_out;
        } else {
            self.reserve_b = new_reserve_in;
            self.reserve_a = new_reserve_out;
        }

        Ok(amount_out)
    }

    fn get_token_out(&self, token_in: AssetId) -> AssetId {
        if self.token_a == token_in {
            self.token_b
        } else {
            self.token_a
        }
    }
}

impl Oxiswap {
    /// Creates a new Oxiswap instance.
    pub fn new(
        address: ContractId,
        token_a: AssetId,
        token_b: AssetId,
        reserve_a: u64,
        reserve_b: u64,
        fee: u64,
    ) -> Self {
        Self {
            address,
            token_a,
            token_b,
            reserve_a,
            reserve_b,
            fee,
        }
    }

    /// Fetches the current pool information from the blockchain.
    pub async fn get_pool_info(&self, wallet: Wallet) -> Result<Oxiswap, AMMError> {
        todo!()
    }

    /// Fetches the current reserves from the blockchain.
    pub async fn get_reserves(&self, wallet: Wallet) -> Result<(u64, u64), AMMError> {
        todo!()
    }

    /// Calculates the amount of tokens received for a given input amount.
    pub fn get_amount_out(
        &self,
        amount_in: u64,
        reserve_in: u64,
        reserve_out: u64,
    ) -> Result<u64, SwapSimulationError> {
        if amount_in == 0 || reserve_in == 0 || reserve_out == 0 {
            return Ok(0);
        }

        let fee_adjustment = 10_000 - (self.fee / 10);
        let amount_in_with_fee = amount_in
            .checked_mul(fee_adjustment)
            .ok_or(SwapSimulationError::Overflow())?;

        let numerator = amount_in_with_fee
            .checked_mul(reserve_out)
            .ok_or(SwapSimulationError::Overflow())?;

        let denominator = reserve_in
            .checked_mul(10_000)
            .and_then(|v| v.checked_add(amount_in_with_fee))
            .ok_or(SwapSimulationError::Overflow())?;

        numerator
            .checked_div(denominator)
            .ok_or(SwapSimulationError::DivisionByZero())
    }
}
