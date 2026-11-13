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
fn deposit_moves_lamports() {
    let mut env = Env::new();
    let client = VaultClient::new();
    initialize_vault(&mut env, &client);

    let deposit_lamports = 1_000_000_000;
    let user_balance_before = account_lamports(&env, &env.user.pubkey());
    let (_, vault) = find_vault_accounts(&env.user.pubkey());
    let deposit_ix = client.deposit(env.user.pubkey(), deposit_lamports);

    let transaction_fee = send_tx_expect_ok(&mut env.svm, deposit_ix, &[&env.user]);

    assert_eq!(account_lamports(&env, &vault), deposit_lamports);
    assert_eq!(
        account_lamports(&env, &env.user.pubkey()),
        user_balance_before - deposit_lamports - transaction_fee
    );
}

#[test]
fn deposit_rejects_below_minimum() {
    let mut env = Env::new();
    let client = VaultClient::new();
    initialize_vault(&mut env, &client);

    let (_, vault) = find_vault_accounts(&env.user.pubkey());
    let deposit_ix = client.deposit(env.user.pubkey(), VaultState::MIN_BALANCE - 1);

    let error_text = send_tx_expect_error(&mut env.svm, deposit_ix, &[&env.user]);

    assert!(
        error_text.contains("Error Code: DepositTooSmall"),
        "{error_text}"
    );
    assert!(env.svm.get_account(&vault).is_none());
}

#[test]
fn deposit_accepts_minimum() {
    let mut env = Env::new();
    let client = VaultClient::new();
    initialize_vault(&mut env, &client);

    let (_, vault) = find_vault_accounts(&env.user.pubkey());
    let deposit_ix = client.deposit(env.user.pubkey(), VaultState::MIN_BALANCE);

    send_tx_expect_ok(&mut env.svm, deposit_ix, &[&env.user]);

    assert_eq!(account_lamports(&env, &vault), VaultState::MIN_BALANCE);
}
