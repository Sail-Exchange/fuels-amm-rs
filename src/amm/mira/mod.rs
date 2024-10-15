pub mod factory;
use super::{consts::U128_0X10000000000000000, AutomatedMarketMaker};
use crate::errors::{AMMError, ArithmeticError, SwapSimulationError};
use async_trait::async_trait;
use fuels::{
    accounts::{impersonated_account::ImpersonatedAccount, wallet::Wallet},
    programs::calls::Execution,
    types::{transaction::TxPolicies, AssetId, ContractId, U256},
};
use mira_v1::interface::{PoolId, PoolMetadata};
use num_bigfloat::BigFloat;
use serde::{Deserialize, Serialize};

/// Represents a Mira pool.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct MiraV1 {
    pub address: ContractId,
    pub pool_id: PoolId,
    pub token_0: AssetId,
    pub token_0_decimals: u8,
    pub token_1: AssetId,
    pub token_1_decimals: u8,
    pub reserve_0: u64,
    pub reserve_1: u64,
    // The different fees (lp_fee_volatile, lp_fee_stable, protocol_fee_volatile, protocol_fee_stable)
    pub fee: (u64, u64, u64, u64),
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
        if self.is_stable {
            let price: U256 = if self.token_0 == base_token {
                self.get_stable_price(
                    U256::from(self.reserve_0),
                    U256::from(self.reserve_1),
                    U256::from(self.token_0_decimals),
                    U256::from(self.token_1_decimals),
                )
            } else {
                self.get_stable_price(
                    U256::from(self.reserve_1),
                    U256::from(self.reserve_0),
                    U256::from(self.token_1_decimals),
                    U256::from(self.token_0_decimals),
                )
            };
            Ok(u256_to_f64(price))
        } else {
            Ok(q64_to_f64(self.calculate_price_64_x_64(base_token)?))
        }
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
                U256::from(self.token_0_decimals),
                U256::from(self.token_1_decimals),
            ))
        } else {
            Ok(self.get_amount_out(
                amount_in,
                U256::from(self.reserve_1),
                U256::from(self.reserve_0),
                U256::from(self.token_1_decimals),
                U256::from(self.token_0_decimals),
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
                U256::from(self.token_0_decimals),
                U256::from(self.token_1_decimals),
            );

            self.reserve_0 += amount_in.as_u64();
            self.reserve_1 -= amount_out.as_u64();

            Ok(amount_out)
        } else {
            let amount_out = self.get_amount_out(
                amount_in,
                U256::from(self.reserve_1),
                U256::from(self.reserve_0),
                U256::from(self.token_1_decimals),
                U256::from(self.token_0_decimals),
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
        pool_id: PoolId,
        token_0: AssetId,
        token_0_decimals: u8,
        token_1: AssetId,
        token_1_decimals: u8,
        reserve_0: u64,
        reserve_1: u64,
        fee: (u64, u64, u64, u64),
        is_stable: bool,
    ) -> Self {
        Self {
            address,
            pool_id,
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
        let address = wallet.address();
        let provider = wallet.provider();
        let simulation_account: ImpersonatedAccount =
            ImpersonatedAccount::new(address.clone(), provider.cloned());
        let mira_contract =
            mira_v1::interface::MiraAmmContract::new(self.address, simulation_account);

        let pool_metadata: PoolMetadata = mira_contract
            .methods()
            .pool_metadata(self.pool_id)
            .with_tx_policies(TxPolicies::default())
            .simulate(Execution::StateReadOnly)
            .await?
            .value
            .unwrap();
        let (lp_fee_volatile, lp_fee_stable, protocol_fee_volatile, protocol_fee_stable) =
            mira_contract
                .methods()
                .fees()
                .with_tx_policies(TxPolicies::default())
                .simulate(Execution::StateReadOnly)
                .await?
                .value;
        let mira_pool = MiraV1 {
            address: self.address,
            pool_id: self.pool_id,
            token_0: self.pool_id.0,
            token_0_decimals: pool_metadata.decimals_0,
            token_1: self.pool_id.1,
            token_1_decimals: pool_metadata.decimals_1,
            reserve_0: pool_metadata.reserve_0,
            reserve_1: pool_metadata.reserve_1,
            fee: (
                lp_fee_volatile,
                lp_fee_stable,
                protocol_fee_volatile,
                protocol_fee_stable,
            ),
            is_stable: self.pool_id.2,
        };
        Ok(mira_pool)
    }

    /// Fetches the current reserves from the blockchain.
    pub async fn get_reserves(&self, wallet: Wallet) -> Result<(u64, u64), AMMError> {
        let address = wallet.address();
        let provider = wallet.provider();
        let simulation_account: ImpersonatedAccount =
            ImpersonatedAccount::new(address.clone(), provider.cloned());
        let mira_contract =
            mira_v1::interface::MiraAmmContract::new(self.address, simulation_account);
        let pool_metadata: PoolMetadata = mira_contract
            .methods()
            .pool_metadata(self.pool_id)
            .with_tx_policies(TxPolicies::default())
            .simulate(Execution::StateReadOnly)
            .await?
            .value
            .unwrap();

        Ok((pool_metadata.reserve_0, pool_metadata.reserve_1))
    }

    /// Calculates the amount received for a given `amount_in` `reserve_in` and `reserve_out`.

    pub fn get_amount_out(
        &self,
        amount_in: U256,
        reserve_in: U256,
        reserve_out: U256,
        decimals_in: U256,
        decimals_out: U256,
    ) -> U256 {
        // Early return if any input is zero
        if amount_in.is_zero() || reserve_in.is_zero() || reserve_out.is_zero() {
            return U256::zero();
        }

        if self.is_stable {
            self.get_stable_amount_out(
                amount_in,
                reserve_in,
                reserve_out,
                decimals_in,
                decimals_out,
            )
        } else {
            self.get_volatile_amount_out(amount_in, reserve_in, reserve_out)
        }
    }

    /// Calculates the output amount for a volatile (constant product) pool.
    fn get_volatile_amount_out(
        &self,
        amount_in: U256,
        reserve_in: U256,
        reserve_out: U256,
    ) -> U256 {
        let fee_numerator = U256::from(10000 - ((self.fee.0 + self.fee.2) / 10));
        let fee_denominator = U256::from(10000);

        let amount_in_with_fee = amount_in * fee_numerator;
        let numerator = amount_in_with_fee * reserve_out;
        let denominator = (reserve_in * fee_denominator) + amount_in_with_fee;

        numerator / denominator
    }

    /// Calculates the output amount for a stable pool.
    fn get_stable_amount_out(
        &self,
        amount_in: U256,
        reserve_in: U256,
        reserve_out: U256,
        decimals_in: U256,
        decimals_out: U256,
    ) -> U256 {
        let xy = self.k(true, reserve_in, reserve_out, decimals_in, decimals_out);
        let amount_in_adjusted = self.adjust(amount_in, decimals_in);
        let reserve_in_adjusted = self.adjust(reserve_in, decimals_in);
        let reserve_out_adjusted = self.adjust(reserve_out, decimals_out);

        let y = reserve_out_adjusted
            - self.y(
                amount_in_adjusted + reserve_in_adjusted,
                xy,
                reserve_out_adjusted,
            );

        self.unadjust(y, decimals_out)
    }

    /// Calculates the invariant k for the pool.
    ///
    /// For stable pools: k = (x^3 * y + y^3 * x) / 10^18
    /// For volatile pools: k = x * y
    fn k(&self, is_stable: bool, x: U256, y: U256, decimals_x: U256, decimals_y: U256) -> U256 {
        if is_stable {
            let x_adjusted = self.adjust(x, decimals_x);
            let y_adjusted = self.adjust(y, decimals_y);

            let a = (x_adjusted * y_adjusted) / self.one_e_18();
            let b = (x_adjusted * x_adjusted) / self.one_e_18()
                + (y_adjusted * y_adjusted) / self.one_e_18();

            (a * b) / self.one_e_18() // (x^3 * y + y^3 * x) / 10^18
        } else {
            x * y // xy >= k
        }
    }

    /// Calculates the y value for the stable swap equation.
    fn y(&self, x_0: U256, xy: U256, y: U256) -> U256 {
        let mut y = y;
        for _ in 0..255 {
            let y_prev = y;
            let k = self.f(x_0, y);

            if k < xy {
                let dy = ((xy - k) * self.one_e_18()) / self.d(x_0, y);
                y += dy;
            } else {
                let dy = ((k - xy) * self.one_e_18()) / self.d(x_0, y);
                y = y.saturating_sub(dy);
            }

            if y > y_prev {
                if y - y_prev <= U256::from(1) {
                    return y;
                }
            } else if y_prev - y <= U256::from(1) {
                return y;
            }
        }
        y
    }

    /// Calculates f(x,y) = x^3*y + y^3*x for the stable swap equation.
    fn f(&self, x: U256, y: U256) -> U256 {
        let x_squared = (x * x) / self.one_e_18();
        let y_squared = (y * y) / self.one_e_18();

        ((x * y_squared) / self.one_e_18()) + ((y * x_squared) / self.one_e_18())
    }

    /// Calculates d(x,y) = 3x^2*y + y^3 for the stable swap equation.
    fn d(&self, x: U256, y: U256) -> U256 {
        let x_squared = (x * x) / self.one_e_18();
        let y_squared = (y * y) / self.one_e_18();

        U256::from(3) * ((x_squared * y) / self.one_e_18()) + y_squared
    }

    /// Adjusts the amount to 18 decimal places for internal calculations.
    fn adjust(&self, amount: U256, decimals: U256) -> U256 {
        amount * self.one_e_18() / U256::from(10).pow(decimals)
    }

    /// Unadjusts the amount from 18 decimal places to the original decimal places.
    fn unadjust(&self, amount: U256, decimals: U256) -> U256 {
        amount * U256::from(10).pow(decimals) / self.one_e_18()
    }

    /// Returns 10^18 as a U256 value.
    fn one_e_18(&self) -> U256 {
        U256::from(10).pow(U256::from(18))
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
    pub fn get_stable_price(
        &self,
        reserve_x: U256,
        reserve_y: U256,
        decimals_x: U256,
        decimals_y: U256,
    ) -> U256 {
        // Adjust reserves to 18 decimal places
        let x = self.adjust(reserve_x, decimals_x);
        let y = self.adjust(reserve_y, decimals_y);

        // Calculate x^3 and y^3
        let x3 = x.pow(U256::from(3)) / self.one_e_18().pow(U256::from(2));
        let y3 = y.pow(U256::from(3)) / self.one_e_18().pow(U256::from(2));

        // Calculate the price using the derivative of the stable curve formula
        let numerator = x3 + self.one_e_18() * x * y;
        let denominator = y3 + self.one_e_18() * x * y;

        // The price is (y^3 + xy) / (x^3 + xy)
        let price = (numerator * self.one_e_18()) / denominator;

        // Adjust the price for the difference in token decimals
        if decimals_x >= decimals_y {
            price * U256::from(10).pow(decimals_x - decimals_y)
        } else {
            price / U256::from(10).pow(decimals_y - decimals_x)
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
pub fn u256_to_f64(value: U256) -> f64 {
    let one_e18 = U256::from(10).pow(U256::from(18));
    let whole_part = (value / one_e18).as_u128() as f64;
    let fractional_part = (value % one_e18).as_u128() as f64 / 1e18;
    whole_part + fractional_part
}
#[allow(unused_imports)]
mod tests {
    use crate::amm::{mira::MiraV1, AutomatedMarketMaker};
    use fuels::types::{AssetId, ContractId};
    use mira_v1::interface::PoolId;

    #[test]
    fn test_calculate_price_edge_case() {
        let token_0 = AssetId::zeroed();
        let token_1 = AssetId::zeroed();
        let x = MiraV1 {
            address: ContractId::zeroed(),
            pool_id: PoolId::default(),
            token_0,
            token_0_decimals: 18,
            token_1,
            token_1_decimals: 9,
            reserve_0: 23595096,
            reserve_1: 15466423,
            fee: (300, 300, 300, 300),
            is_stable: false,
        };

        assert!(x.calculate_price(token_0, AssetId::default()).unwrap() != 0.0);
        assert!(x.calculate_price(token_1, AssetId::default()).unwrap() != 0.0);
    }
}
