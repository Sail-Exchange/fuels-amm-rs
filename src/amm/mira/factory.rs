use async_trait::async_trait;
use fuel_storage::StorageRead;
use fuels::{
    accounts::{impersonated_account::ImpersonatedAccount, wallet::Wallet},
    programs::calls::Execution,
    types::{transaction::TxPolicies, ContractId, U256},
};
use serde::{Deserialize, Serialize};

use crate::{
    amm::{factory::AutomatedMarketMakerFactory, AMM},
    errors::AMMError,
};
use mira_v1::interface::MiraAmmContract;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct MiraV1Factory {
    pub contract_id: ContractId,
    pub creation_block: u64,
}

#[async_trait]
impl AutomatedMarketMakerFactory for MiraV1Factory {
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

impl MiraV1Factory {
    pub fn new(contract_id: ContractId, creation_block: u64) -> MiraV1Factory {
        MiraV1Factory {
            contract_id,
            creation_block,
        }
    }
    pub async fn get_all_pairs_via_batched_calls(
        &self,
        wallet: Wallet,
    ) -> Result<Vec<AMM>, AMMError> {
        // Create Factory
        let address = wallet.address();
        let provider = wallet.provider();
        let simulation_account: ImpersonatedAccount =
            ImpersonatedAccount::new(address.clone(), provider.cloned());
        let mira_contract = MiraAmmContract::new(self.contract_id, simulation_account);

        // Get the number of pools
        let number_of_pools = mira_contract
            .methods()
            .total_assets()
            .with_tx_policies(TxPolicies::default())
            .simulate(Execution::StateReadOnly)
            .await?
            .value;
        let mut pairs: Vec<AMM> = vec![];
        let step = 766;
        // Check to see if step size is greater than number of pairs and set step accordingly
        let mut idx_from = U256::zero();
        let mut idx_to = if step > number_of_pools {
            U256::from(number_of_pools)
        } else {
            U256::from(step)
        };

        for _ in (0..number_of_pools).step_by(step.try_into().unwrap()) {
            // TODO: Append the pairs
            idx_from = idx_to;

            if idx_to + U256::from(step) > U256::from(number_of_pools) {
                idx_to = U256::from(number_of_pools) - U256::from_little_endian(&[1, 0, 0, 0])
            } else {
                idx_to += U256::from(step);
            }
        }
        todo!()
    }
}

// impl StorageRead for MiraAmmContract {
//     fn read(&self, key: &Type::Key, buf: &mut [u8]) -> Result<Option<usize>, Self::Error> {
//         todo!()
//     }

//     fn read_alloc(&self, key: &Type::Key) -> Result<Option<Vec<u8>>, Self::Error> {
//         todo!()
//     }
// }
