mod common;

use {
    common::{
        account_lamports, client::VaultClient, find_vault_accounts, initialize_vault,
        send_tx_expect_error, send_tx_expect_ok, Env,
    },
    solana_signer::Signer,
    vault::state::vault_state::VaultState,
};

fn deposit(env: &mut Env, client: &VaultClient, amount: u64) {
    let deposit_ix = client.deposit(env.user.pubkey(), amount);
    send_tx_expect_ok(&mut env.svm, deposit_ix, &[&env.user]);
}

#[test]
fn withdraw_moves_lamports() {
    let mut env = Env::new();
    let client = VaultClient::new();
    initialize_vault(&mut env, &client);
    let deposit_lamports = 1_000_000_000;
    deposit(&mut env, &client, deposit_lamports);

    let withdraw_lamports = 250_000_000;
    let user_balance_before = account_lamports(&env, &env.user.pubkey());
    let (_, vault) = find_vault_accounts(&env.user.pubkey());
    let withdraw_ix = client.withdraw(env.user.pubkey(), withdraw_lamports);

    let transaction_fee = send_tx_expect_ok(&mut env.svm, withdraw_ix, &[&env.user]);

    assert_eq!(
        account_lamports(&env, &vault),
        deposit_lamports - withdraw_lamports
    );
    assert_eq!(
        account_lamports(&env, &env.user.pubkey()),
        user_balance_before + withdraw_lamports - transaction_fee
    );
}

#[test]
fn withdraw_rejects_zero_amount() {
    let mut env = Env::new();
    let client = VaultClient::new();
    initialize_vault(&mut env, &client);
    deposit(&mut env, &client, VaultState::MIN_BALANCE);

    let (_, vault) = find_vault_accounts(&env.user.pubkey());
    let vault_balance_before = account_lamports(&env, &vault);
    let withdraw_ix = client.withdraw(env.user.pubkey(), 0);

    let error_text = send_tx_expect_error(&mut env.svm, withdraw_ix, &[&env.user]);

    assert!(
        error_text.contains("Error Code: ZeroAmount"),
        "{error_text}"
    );
    assert_eq!(account_lamports(&env, &vault), vault_balance_before);
}

#[test]
fn withdraw_rejects_insufficient_funds() {
    let mut env = Env::new();
    let client = VaultClient::new();
    initialize_vault(&mut env, &client);
    deposit(&mut env, &client, VaultState::MIN_BALANCE);

    let (_, vault) = find_vault_accounts(&env.user.pubkey());
    let withdraw_ix = client.withdraw(env.user.pubkey(), VaultState::MIN_BALANCE + 1);

    let error_text = send_tx_expect_error(&mut env.svm, withdraw_ix, &[&env.user]);

    assert!(
        error_text.contains("Error Code: InsufficientFunds"),
        "{error_text}"
    );
    assert_eq!(account_lamports(&env, &vault), VaultState::MIN_BALANCE);
}

#[test]
fn withdraw_rejects_below_minimum_balance() {
    let mut env = Env::new();
    let client = VaultClient::new();
    initialize_vault(&mut env, &client);
    deposit(&mut env, &client, VaultState::MIN_BALANCE * 2);

    let (_, vault) = find_vault_accounts(&env.user.pubkey());
    let withdraw_ix = client.withdraw(env.user.pubkey(), VaultState::MIN_BALANCE + 1);

    let error_text = send_tx_expect_error(&mut env.svm, withdraw_ix, &[&env.user]);

    assert!(
        error_text.contains("Error Code: BelowMinBalance"),
        "{error_text}"
    );
    assert_eq!(account_lamports(&env, &vault), VaultState::MIN_BALANCE * 2);
}

#[test]
fn withdraw_accepts_remaining_minimum() {
    let mut env = Env::new();
    let client = VaultClient::new();
    initialize_vault(&mut env, &client);
    deposit(&mut env, &client, VaultState::MIN_BALANCE * 2);

    let (_, vault) = find_vault_accounts(&env.user.pubkey());
    let withdraw_ix = client.withdraw(env.user.pubkey(), VaultState::MIN_BALANCE);

    send_tx_expect_ok(&mut env.svm, withdraw_ix, &[&env.user]);

    assert_eq!(account_lamports(&env, &vault), VaultState::MIN_BALANCE);
}

#[test]
fn withdraw_accepts_full_balance() {
    let mut env = Env::new();
    let client = VaultClient::new();
    initialize_vault(&mut env, &client);
    let deposit_lamports = 1_000_000_000;
    deposit(&mut env, &client, deposit_lamports);

    let user_balance_before = account_lamports(&env, &env.user.pubkey());
    let (_, vault) = find_vault_accounts(&env.user.pubkey());
    let withdraw_ix = client.withdraw(env.user.pubkey(), deposit_lamports);

    let transaction_fee = send_tx_expect_ok(&mut env.svm, withdraw_ix, &[&env.user]);

    assert_eq!(account_lamports(&env, &vault), 0);
    assert_eq!(
        account_lamports(&env, &env.user.pubkey()),
        user_balance_before + deposit_lamports - transaction_fee
    );
}
