mod common;

use {
    common::{
        account_lamports, client::VaultClient, find_vault_accounts, initialize_vault,
        send_tx_expect_error, send_tx_expect_ok, Env,
    },
    solana_signer::Signer,
    vault::state::vault_state::VaultState,
};

#[test]
fn close_returns_balance_and_rent() {
    let mut env = Env::new();
    let client = VaultClient::new();
    initialize_vault(&mut env, &client);

    let deposit_lamports = 1_000_000_000;
    let deposit_ix = client.deposit(env.user.pubkey(), deposit_lamports);
    send_tx_expect_ok(&mut env.svm, deposit_ix, &[&env.user]);

    let (vault_state, vault) = find_vault_accounts(&env.user.pubkey());
    let vault_balance = account_lamports(&env, &vault);
    let vault_state_rent = account_lamports(&env, &vault_state);
    let user_balance_before = account_lamports(&env, &env.user.pubkey());
    let close_ix = client.close(env.user.pubkey());

    let transaction_fee = send_tx_expect_ok(&mut env.svm, close_ix, &[&env.user]);

    assert!(env.svm.get_account(&vault_state).is_none());
    assert!(env.svm.get_account(&vault).is_none());
    assert_eq!(
        account_lamports(&env, &env.user.pubkey()),
        user_balance_before + vault_balance + vault_state_rent - transaction_fee
    );
}

#[test]
fn close_round_trip() {
    let mut env = Env::new();
    let client = VaultClient::new();
    let user_balance_before = account_lamports(&env, &env.user.pubkey());
    let (vault_state, vault) = find_vault_accounts(&env.user.pubkey());

    let initialize_ix = client.initialize(env.user.pubkey());
    let initialize_fee = send_tx_expect_ok(&mut env.svm, initialize_ix, &[&env.user]);

    let first_deposit_ix = client.deposit(env.user.pubkey(), 1_000_000_000);
    let first_deposit_fee = send_tx_expect_ok(&mut env.svm, first_deposit_ix, &[&env.user]);

    let withdraw_ix = client.withdraw(env.user.pubkey(), 400_000_000);
    let withdraw_fee = send_tx_expect_ok(&mut env.svm, withdraw_ix, &[&env.user]);

    let second_deposit_ix = client.deposit(env.user.pubkey(), 500_000_000);
    let second_deposit_fee = send_tx_expect_ok(&mut env.svm, second_deposit_ix, &[&env.user]);

    let close_ix = client.close(env.user.pubkey());
    let close_fee = send_tx_expect_ok(&mut env.svm, close_ix, &[&env.user]);

    assert!(env.svm.get_account(&vault_state).is_none());
    assert!(env.svm.get_account(&vault).is_none());
    assert_eq!(
        account_lamports(&env, &env.user.pubkey()),
        user_balance_before
            - initialize_fee
            - first_deposit_fee
            - withdraw_fee
            - second_deposit_fee
            - close_fee
    );
}

#[test]
fn close_accepts_empty_vault() {
    let mut env = Env::new();
    let client = VaultClient::new();
    let user_balance_before = account_lamports(&env, &env.user.pubkey());
    let (vault_state, vault) = find_vault_accounts(&env.user.pubkey());

    let initialize_ix = client.initialize(env.user.pubkey());
    let initialize_fee = send_tx_expect_ok(&mut env.svm, initialize_ix, &[&env.user]);
    let close_ix = client.close(env.user.pubkey());
    let close_fee = send_tx_expect_ok(&mut env.svm, close_ix, &[&env.user]);

    assert!(env.svm.get_account(&vault_state).is_none());
    assert!(env.svm.get_account(&vault).is_none());
    assert_eq!(
        account_lamports(&env, &env.user.pubkey()),
        user_balance_before - initialize_fee - close_fee
    );
}

#[test]
fn deposit_rejects_closed_vault() {
    let mut env = Env::new();
    let client = VaultClient::new();
    initialize_vault(&mut env, &client);
    let close_ix = client.close(env.user.pubkey());
    send_tx_expect_ok(&mut env.svm, close_ix, &[&env.user]);

    let deposit_ix = client.deposit(env.user.pubkey(), VaultState::MIN_BALANCE);
    let error_text = send_tx_expect_error(&mut env.svm, deposit_ix, &[&env.user]);

    assert!(
        error_text.contains("Error Code: AccountNotInitialized"),
        "{error_text}"
    );
}
