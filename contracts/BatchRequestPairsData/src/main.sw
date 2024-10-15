script;
use interfaces::mira_amm::MiraAMM;
use std::storage::storage_api::read;
configurable {
    MIRA_CONTRACT_ID: b256 = 0xd5a716d967a9137222219657d7877bd8c79c64e1edb5de9f2901c98ebe74da80,
}
fn main() {
    let contract_address = MIRA_CONTRACT_ID;
    let mira_amm = abi(MiraAMM, contract_address);
    let contract_id = ContractId::from(contract_address);
    let pools = mira_amm;
    log(pools);
}
