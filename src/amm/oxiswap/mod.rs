use async_trait::async_trait;
use fuels::{
    accounts::wallet::Wallet,
    types::{AssetId, ContractId, U256},
};
use serde::{Deserialize, Serialize};

use super::AutomatedMarketMaker;
use crate::errors::{AMMError, ArithmeticError, SwapSimulationError};

/// Represents an Oxiswap pool.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct Oxiswap {
    pub address: ContractId,
    pub token_0: AssetId,
    pub token_0_decimals: u8,
    pub token_1: AssetId,
    pub token_1_decimals: u8,
    pub reserve_0: u64,
    pub reserve_1: u64,
    pub fee: u64,
}

#[async_trait]
impl AutomatedMarketMaker for Oxiswap {
    fn address(&self) -> ContractId {
        self.address
    }

    /// Synchronizes the AMM's state with the blockchain.
    async fn sync(&mut self, wallet: Wallet) -> Result<(), AMMError> {
        let (reserve_0, reserve_1) = self.get_reserves(wallet).await?;
        self.reserve_0 = reserve_0;
        self.reserve_1 = reserve_1;
        Ok(())
    }

    fn tokens(&self) -> Vec<AssetId> {
        vec![self.token_0, self.token_1]
    }

    /// Calculates the price of the base token in terms of the other token.
    fn calculate_price(
        &self,
        base_token: AssetId,
        _quote_token: AssetId,
    ) -> Result<f64, ArithmeticError> {
        let (reserve_in, reserve_out) = if base_token == self.token_0 {
            (self.reserve_0, self.reserve_1)
        } else {
            (self.reserve_1, self.reserve_0)
        };

        let some_price = reserve_out.checked_div(reserve_in);
        Ok(some_price.unwrap() as f64)
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
    fn simulate_swap(
        &self,
        base_token: AssetId,
        _quote_token: AssetId,
        amount_in: U256,
    ) -> Result<U256, SwapSimulationError> {
        if self.token_0 == base_token {
            Ok(self.get_amount_out(
                amount_in,
                U256::from(self.reserve_0),
                U256::from(self.reserve_1),
            ))
        } else {
            Ok(self.get_amount_out(
                amount_in,
                U256::from(self.reserve_1),
                U256::from(self.reserve_0),
            ))
        }
    }

    /// Simulates a swap and updates the AMM's state.
    fn simulate_swap_mut(
        &mut self,
        base_token: AssetId,
        _quote_token: AssetId,
        amount_in: U256,
    ) -> Result<U256, SwapSimulationError> {
        if self.token_0 == base_token {
            let amount_out = self.get_amount_out(
                amount_in,
                U256::from(self.reserve_0),
                U256::from(self.reserve_1),
            );

            self.reserve_0 += amount_in.as_u64();
            self.reserve_1 -= amount_out.as_u64();

            Ok(amount_out)
        } else {
            let amount_out = self.get_amount_out(
                amount_in,
                U256::from(self.reserve_1),
                U256::from(self.reserve_0),
            );

            self.reserve_0 -= amount_out.as_u64();
            self.reserve_1 += amount_in.as_u64();

            Ok(amount_out)
        }
    }

    fn get_token_out(&self, token_in: AssetId) -> AssetId {
        if self.token_0 == token_in {
            self.token_1
        } else {
            self.token_0
        }
    }
}

impl Oxiswap {
    /// Creates a new Oxiswap instance.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        address: ContractId,
        token_0: AssetId,
        token_0_decimals: u8,
        token_1: AssetId,
        token_1_decimals: u8,
        reserve_0: u64,
        reserve_1: u64,
        fee: u64,
    ) -> Self {
        Self {
            address,
            token_0,
            token_0_decimals,
            token_1,
            token_1_decimals,
            reserve_0,
            reserve_1,
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

    /// Calculates the amount received for a given `amount_in` `reserve_in` and `reserve_out`.
    pub fn get_amount_out(&self, amount_in: U256, reserve_in: U256, reserve_out: U256) -> U256 {
        if amount_in.is_zero() || reserve_in.is_zero() || reserve_out.is_zero() {
            return U256::zero();
        }
        let fee = (10000 - (self.fee / 10)) / 10; //Fee of 300 => (10,000 - 30) / 10  = 997
        let amount_in_with_fee = amount_in * U256::from(fee);
        let numerator = amount_in_with_fee * reserve_out;
        let denominator = reserve_in * U256::from(1000) + amount_in_with_fee;

        numerator / denominator
    }
}
