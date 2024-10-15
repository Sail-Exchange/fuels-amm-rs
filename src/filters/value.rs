use crate::amm::AMM;
pub fn filter_amms_with_empty_reserves(amms: Vec<AMM>) -> Vec<AMM> {
    let mut cleaned_amms = vec![];

    for amm in amms.into_iter() {
        match amm {
            AMM::MiraV1(mira_v1_pool) => {
                if mira_v1_pool.reserve_0 == 0 && mira_v1_pool.reserve_1 == 0 {
                    cleaned_amms.push(amm)
                }
            }
            // TODO: Fix the a b vs 0 1 when oxiswap gets merged
            AMM::Oxiswap(oxiswap_pool) => {
                if oxiswap_pool.reserve_a == 0 && oxiswap_pool.reserve_b == 0 {
                    cleaned_amms.push(amm)
                }
            }
        }
    }

    cleaned_amms
}
