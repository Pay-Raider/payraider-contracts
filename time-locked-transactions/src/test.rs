#![cfg(test)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(clippy::panic)]

use soroban_sdk::{
    testutils::{Address as _, Ledger},
    token, Address, Env,
};

use crate::{TimeLockedTransactionsContract, TimeLockedTransactionsContractClient, TransferState};

fn setup(env: &Env) -> (TimeLockedTransactionsContractClient, Address, Address) {
    let admin = Address::generate(env);
    let contract_id = env.register(TimeLockedTransactionsContract, ());
    let client = TimeLockedTransactionsContractClient::new(env, &contract_id);
    client.initialize(&admin);
    (client, contract_id, admin)
}

fn advance_time(env: &Env, seconds: u64) {
    env.ledger().with_mut(|li| {
        li.timestamp += seconds;
    });
}

#[test]
fn test_initialize() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _, admin) = setup(&env);

    assert_eq!(client.get_admin(), admin);
    assert!(!client.is_paused());
    assert_eq!(client.get_transfer_count(), 0);
}

#[test]
#[should_panic]
fn test_double_initialize_fails() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _, admin) = setup(&env);
    client.initialize(&admin);
}

#[test]
fn test_schedule_transfer() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _, admin) = setup(&env);
    let sender = Address::generate(&env);
    let recipient = Address::generate(&env);

    let token_addr = env.register_stellar_asset_contract_v2(admin.clone()).address();
    let asset_client = token::StellarAssetClient::new(&env, &token_addr);
    asset_client.mint(&sender, &10_000_000);

    let unlock_time = env.ledger().timestamp() + 3600;
    let amount: i128 = 1_000_000;

    let tx_id = client.schedule_transfer(&sender, &recipient, &token_addr, &amount, &unlock_time);
    assert_eq!(tx_id, 1);
    assert_eq!(client.get_transfer_count(), 1);

    let transfer = client.get_transfer(&tx_id);
    assert_eq!(transfer.sender, sender);
    assert_eq!(transfer.recipient, recipient);
    assert_eq!(transfer.amount, amount);
    assert_eq!(transfer.unlock_time, unlock_time);
    assert_eq!(transfer.state, TransferState::Pending);

    // Tokens are now held by the contract
    let token_client = token::Client::new(&env, &token_addr);
    assert_eq!(token_client.balance(&sender), 9_000_000);
}

#[test]
fn test_execute_after_unlock() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _, admin) = setup(&env);
    let sender = Address::generate(&env);
    let recipient = Address::generate(&env);

    let token_addr = env.register_stellar_asset_contract_v2(admin.clone()).address();
    let asset_client = token::StellarAssetClient::new(&env, &token_addr);
    let amount: i128 = 2_000_000;
    asset_client.mint(&sender, &amount);

    let unlock_time = env.ledger().timestamp() + 3600;
    let tx_id = client.schedule_transfer(&sender, &recipient, &token_addr, &amount, &unlock_time);

    // Advance past unlock
    advance_time(&env, 3601);
    client.execute_transfer(&sender, &tx_id);

    let transfer = client.get_transfer(&tx_id);
    assert_eq!(transfer.state, TransferState::Executed);

    let token_client = token::Client::new(&env, &token_addr);
    assert_eq!(token_client.balance(&recipient), amount);
}

#[test]
fn test_recipient_can_execute_after_unlock() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _, admin) = setup(&env);
    let sender = Address::generate(&env);
    let recipient = Address::generate(&env);

    let token_addr = env.register_stellar_asset_contract_v2(admin.clone()).address();
    let asset_client = token::StellarAssetClient::new(&env, &token_addr);
    let amount: i128 = 1_000_000;
    asset_client.mint(&sender, &amount);

    let unlock_time = env.ledger().timestamp() + 100;
    let tx_id = client.schedule_transfer(&sender, &recipient, &token_addr, &amount, &unlock_time);

    advance_time(&env, 200);
    client.execute_transfer(&recipient, &tx_id);

    assert_eq!(client.get_transfer(&tx_id).state, TransferState::Executed);
}

#[test]
#[should_panic]
fn test_execute_before_unlock_fails() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _, admin) = setup(&env);
    let sender = Address::generate(&env);
    let recipient = Address::generate(&env);

    let token_addr = env.register_stellar_asset_contract_v2(admin.clone()).address();
    let asset_client = token::StellarAssetClient::new(&env, &token_addr);
    asset_client.mint(&sender, &1_000_000);

    let unlock_time = env.ledger().timestamp() + 3600;
    let tx_id = client.schedule_transfer(&sender, &recipient, &token_addr, &1_000_000, &unlock_time);

    // Not yet past unlock — should panic
    client.execute_transfer(&sender, &tx_id);
}

#[test]
fn test_cancel_transfer() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _, admin) = setup(&env);
    let sender = Address::generate(&env);
    let recipient = Address::generate(&env);

    let token_addr = env.register_stellar_asset_contract_v2(admin.clone()).address();
    let asset_client = token::StellarAssetClient::new(&env, &token_addr);
    let amount: i128 = 1_500_000;
    asset_client.mint(&sender, &amount);

    let initial_balance = token::Client::new(&env, &token_addr).balance(&sender);

    let unlock_time = env.ledger().timestamp() + 7200;
    let tx_id = client.schedule_transfer(&sender, &recipient, &token_addr, &amount, &unlock_time);

    // Sender cancels before unlock — tokens returned
    client.cancel_transfer(&sender, &tx_id);

    assert_eq!(client.get_transfer(&tx_id).state, TransferState::Cancelled);
    let token_client = token::Client::new(&env, &token_addr);
    assert_eq!(token_client.balance(&sender), initial_balance);
}

#[test]
fn test_admin_can_cancel() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _, admin) = setup(&env);
    let sender = Address::generate(&env);
    let recipient = Address::generate(&env);

    let token_addr = env.register_stellar_asset_contract_v2(admin.clone()).address();
    let asset_client = token::StellarAssetClient::new(&env, &token_addr);
    let amount: i128 = 1_000_000;
    asset_client.mint(&sender, &amount);

    let unlock_time = env.ledger().timestamp() + 3600;
    let tx_id = client.schedule_transfer(&sender, &recipient, &token_addr, &amount, &unlock_time);

    client.cancel_transfer(&admin, &tx_id);
    assert_eq!(client.get_transfer(&tx_id).state, TransferState::Cancelled);
}

#[test]
#[should_panic]
fn test_unauthorized_cancel_fails() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _, admin) = setup(&env);
    let sender = Address::generate(&env);
    let recipient = Address::generate(&env);
    let outsider = Address::generate(&env);

    let token_addr = env.register_stellar_asset_contract_v2(admin.clone()).address();
    let asset_client = token::StellarAssetClient::new(&env, &token_addr);
    asset_client.mint(&sender, &1_000_000);

    let unlock_time = env.ledger().timestamp() + 3600;
    let tx_id = client.schedule_transfer(&sender, &recipient, &token_addr, &1_000_000, &unlock_time);

    client.cancel_transfer(&outsider, &tx_id); // should panic
}

#[test]
#[should_panic]
fn test_execute_cancelled_transfer_fails() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _, admin) = setup(&env);
    let sender = Address::generate(&env);
    let recipient = Address::generate(&env);

    let token_addr = env.register_stellar_asset_contract_v2(admin.clone()).address();
    let asset_client = token::StellarAssetClient::new(&env, &token_addr);
    asset_client.mint(&sender, &1_000_000);

    let unlock_time = env.ledger().timestamp() + 3600;
    let tx_id = client.schedule_transfer(&sender, &recipient, &token_addr, &1_000_000, &unlock_time);
    client.cancel_transfer(&sender, &tx_id);

    advance_time(&env, 7200);
    client.execute_transfer(&sender, &tx_id); // already cancelled — should panic
}

#[test]
fn test_pause_prevents_scheduling() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _, admin) = setup(&env);
    let sender = Address::generate(&env);
    let recipient = Address::generate(&env);

    let token_addr = env.register_stellar_asset_contract_v2(admin.clone()).address();

    client.pause(&admin);
    assert!(client.is_paused());

    let unlock_time = env.ledger().timestamp() + 3600;
    let result = client.try_schedule_transfer(&sender, &recipient, &token_addr, &1_000_000, &unlock_time);
    assert!(result.is_err());

    client.unpause(&admin);
    assert!(!client.is_paused());
}

#[test]
#[should_panic]
fn test_invalid_unlock_time_fails() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _, admin) = setup(&env);
    let sender = Address::generate(&env);
    let recipient = Address::generate(&env);

    let token_addr = env.register_stellar_asset_contract_v2(admin.clone()).address();
    let asset_client = token::StellarAssetClient::new(&env, &token_addr);
    asset_client.mint(&sender, &1_000_000);

    // Unlock time in the past — should panic
    let past_unlock = env.ledger().timestamp().saturating_sub(1);
    client.schedule_transfer(&sender, &recipient, &token_addr, &1_000_000, &past_unlock);
}
