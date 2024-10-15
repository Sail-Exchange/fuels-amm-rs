use fuels::types::{AssetId, ContractId};

use crate::amm::{AutomatedMarketMaker, AMM};

use std::collections::HashSet;

/// Filters out AMMs that contain a blacklisted token.
pub fn filter_blacklisted_tokens(amms: Vec<AMM>, blacklisted_addresses: Vec<AssetId>) -> Vec<AMM> {
    let mut filtered_pools = vec![];
    let blacklist: HashSet<AssetId> = blacklisted_addresses.into_iter().collect();

    for amm in amms {
        let mut blacklisted_token_in_amm = false;
        for token in amm.tokens() {
            if blacklist.contains(&token) {
                blacklisted_token_in_amm = true;
                break;
            }
        }

        if !blacklisted_token_in_amm {
            filtered_pools.push(amm);
        }
    }

    filtered_pools
}

/// Filters out AMMs where the AMM address is a blacklisted address.
pub fn filter_blacklisted_amms(amms: Vec<AMM>, blacklisted_addresses: Vec<ContractId>) -> Vec<AMM> {
    let mut filtered_amms = vec![];
    let blacklist: HashSet<ContractId> = blacklisted_addresses.into_iter().collect();

    for amm in amms {
        if !blacklist.contains(&amm.address()) {
            filtered_amms.push(amm);
        }
    }

    filtered_amms
}
