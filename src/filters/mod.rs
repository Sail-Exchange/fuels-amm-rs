use fuels::types::AssetId;

use crate::amm::AMM;

pub mod address;
pub mod value;

pub fn filter_empty_amms(amms: Vec<AMM>) -> Vec<AMM> {
    let mut cleaned_amms = vec![];

    for amm in amms.into_iter() {
        match amm {
            AMM::MiraV1(mira_v1_pool) => {
                if mira_v1_pool.token_0 == AssetId::zeroed()
                    && mira_v1_pool.token_1 == AssetId::zeroed()
                {
                    cleaned_amms.push(amm)
                }
            }
            // TODO: Fix the a b vs 0 1 when oxiswap gets merged
            AMM::Oxiswap(oxiswap_pool) => {
                if oxiswap_pool.token_a == AssetId::zeroed()
                    && oxiswap_pool.token_b == AssetId::zeroed()
                {
                    cleaned_amms.push(amm)
                }
            }
        }
    }

    cleaned_amms
}
