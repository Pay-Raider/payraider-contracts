use soroban_sdk::{symbol_short, Address, Env};

pub fn emit_initialized(env: &Env, threshold: u32) {
    env.events().publish((symbol_short!("MSW_INI"),), threshold);
}

pub fn emit_tx_proposed(env: &Env, tx_id: u64, proposer: Address, amount: i128) {
    env.events()
        .publish((symbol_short!("MSW_PRP"),), (tx_id, proposer, amount));
}

pub fn emit_tx_confirmed(env: &Env, tx_id: u64, signer: Address) {
    env.events()
        .publish((symbol_short!("MSW_CNF"),), (tx_id, signer));
}

pub fn emit_confirmation_revoked(env: &Env, tx_id: u64, signer: Address) {
    env.events()
        .publish((symbol_short!("MSW_RVK"),), (tx_id, signer));
}

pub fn emit_tx_executed(env: &Env, tx_id: u64, executor: Address, amount: i128) {
    env.events()
        .publish((symbol_short!("MSW_EXE"),), (tx_id, executor, amount));
}

pub fn emit_tx_cancelled(env: &Env, tx_id: u64, cancelled_by: Address) {
    env.events()
        .publish((symbol_short!("MSW_CAN"),), (tx_id, cancelled_by));
}
