pub mod factory;
use super::{consts::U128_0X10000000000000000, AutomatedMarketMaker};
use crate::errors::{AMMError, ArithmeticError, SwapSimulationError};
use async_trait::async_trait;
use fuels::{
    accounts::wallet::Wallet,
    types::{AssetId, ContractId, U256},
};
use num_bigfloat::BigFloat;
use serde::{Deserialize, Serialize};

/// Represents a Mira pool.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct MiraV1 {
    pub address: ContractId,
    pub token_0: AssetId,
    pub token_0_decimals: u8,
    pub token_1: AssetId,
    pub token_1_decimals: u8,
    pub reserve_0: u64,
    pub reserve_1: u64,
    pub fee: u64,
    pub is_stable: bool,
}

#[async_trait]
impl AutomatedMarketMaker for MiraV1 {
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

    //TODO: Handle price calculations for stable swaps
    /// Calculates the price of the base token in terms of the other token.
    fn calculate_price(
        &self,
        base_token: AssetId,
        _quote_token: AssetId,
    ) -> Result<f64, ArithmeticError> {
        Ok(q64_to_f64(self.calculate_price_64_x_64(base_token)?))
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

impl MiraV1 {
    /// Creates a new Mira instance.
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
        is_stable: bool,
    ) -> Self {
        Self {
            address,
            token_0,
            token_1,
            reserve_0,
            reserve_1,
            fee,
            is_stable,
            token_0_decimals,
            token_1_decimals,
        }
    }

    /// Fetches the current pool information from the blockchain.
    pub async fn get_pool_info(&self, wallet: Wallet) -> Result<MiraV1, AMMError> {
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

    /// Calculates the price of the base token in terms of the quote token.
    ///
    /// Returned as a Q64 fixed point number.
    pub fn calculate_price_64_x_64(&self, base_token: AssetId) -> Result<u128, ArithmeticError> {
        let decimal_shift = self.token_0_decimals as i8 - self.token_1_decimals as i8;

        let (r_a, r_1) = if decimal_shift < 0 {
            (
                U256::from(self.reserve_0)
                    * U256::from(10u128.pow(decimal_shift.unsigned_abs() as u32)),
                U256::from(self.reserve_1),
            )
        } else {
            (
                U256::from(self.reserve_0),
                U256::from(self.reserve_1) * U256::from(10u128.pow(decimal_shift as u32)),
            )
        };

        if base_token == self.token_0 {
            if r_a.is_zero() {
                Ok(U256::max_value().as_u128())
            } else {
                div_uu(r_1, r_a)
            }
        } else if r_1.is_zero() {
            Ok(U256::max_value().as_u128())
        } else {
            div_uu(r_a, r_1)
        }
    }
}

pub fn div_uu(x: U256, y: U256) -> Result<u128, ArithmeticError> {
    if !y.is_zero() {
        let mut answer;

        if x <= U256::max_value() {
            answer = (x << U256::from(64)) / y;
        } else {
            let mut msb = U256::from(192);
            let mut xc = x >> U256::from(192);

            // TODO: Figure out how to do the little endian conversion. Might be worth using primitive types crate rather than Fuel U256
            if xc >= U256::from_dec_str("4294967296").unwrap() {
                xc >>= U256::from(32);
                msb += U256::from(32);
            }

            if xc >= U256::from(65536) {
                xc >>= U256::from(16);
                msb += U256::from(16);
            }

            if xc >= U256::from(256) {
                xc >>= U256::from(8);
                msb += U256::from(8);
            }

            if xc >= U256::from(16) {
                xc >>= U256::from(4);
                msb += U256::from(4);
            }

            if xc >= U256::from(4) {
                xc >>= U256::from(2);
                msb += U256::from(2);
            }

            if xc >= U256::from(2) {
                msb += U256::one();
            }

            answer = (x << (U256::from(255) - msb))
                / (((y - U256::one()) >> (msb - U256::from(191))) + U256::one());
        }

        if answer > U256::max_value() {
            return Ok(0);
        }

        let hi = answer * (y >> U256::from(128));
        let mut lo = answer * (y & U256::max_value());

        let mut xh = x >> U256::from(192);
        let mut xl = x << U256::from(64);

        if xl < lo {
            xh -= U256::one();
        }

        xl = xl.overflowing_sub(lo).0;
        lo = hi << U256::from(128);

        if xl < lo {
            xh -= U256::one();
        }

        xl = xl.overflowing_sub(lo).0;

        if xh != hi >> U256::from(128) {
            return Err(ArithmeticError::RoundingError);
        }

        answer += xl / y;

        if answer > U256::max_value() {
            return Ok(0_u128);
        }

        Ok(answer.as_u128())
    } else {
        Err(ArithmeticError::YIsZero)
    }
}

/// Converts a Q64 fixed point to a Q16 fixed point -> f64
pub fn q64_to_f64(x: u128) -> f64 {
    BigFloat::from(x)
        .div(&BigFloat::from(U128_0X10000000000000000))
        .to_f64()
}

#[allow(unused_imports)]
mod tests {
    use crate::amm::{mira::MiraV1, AutomatedMarketMaker};
    use fuels::types::{AssetId, ContractId};

    #[test]
    fn test_calculate_price_edge_case() {
        let token_0 = AssetId::zeroed();
        let token_1 = AssetId::zeroed();
        let x = MiraV1 {
            address: ContractId::zeroed(),
            token_0,
            token_0_decimals: 18,
            token_1,
            token_1_decimals: 9,
            reserve_0: 23595096,
            reserve_1: 15466423,
            fee: 300,
            is_stable: false,
        };

        assert!(x.calculate_price(token_0, AssetId::default()).unwrap() != 0.0);
        assert!(x.calculate_price(token_1, AssetId::default()).unwrap() != 0.0);
    }
}
