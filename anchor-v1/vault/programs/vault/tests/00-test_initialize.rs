mod common;

use {
    common::{client::VaultClient, initialize_vault, send_tx_expect_error, Env},
    solana_signer::Signer,
    vault::state::vault_state::VaultState,
};

#[test]
fn initialize_stores_bumps_and_leaves_vault_uncreated() {
    let mut env = Env::new();
    let client = VaultClient::new();
    let (vault_state, state_bump) = VaultState::find_pda(&env.user.pubkey());
    let (vault, vault_bump) = VaultState::find_vault_pda(&vault_state);

    initialize_vault(&mut env, &client);

    let vault_state_account: VaultState = common::load(&env, &vault_state);
    assert_eq!(vault_state_account.state_bump, state_bump);
    assert_eq!(vault_state_account.vault_bump, vault_bump);
    assert!(env.svm.get_account(&vault).is_none());
}

#[test]
fn initialize_accepts_vault_state_with_existing_lamports() {
    let mut env = Env::new();
    let client = VaultClient::new();
    let (vault_state, state_bump) = VaultState::find_pda(&env.user.pubkey());
    let (vault, vault_bump) = VaultState::find_vault_pda(&vault_state);
    let rent_exempt_lamports_for_zero_bytes = 890_880;

    env.svm
        .airdrop(&vault_state, rent_exempt_lamports_for_zero_bytes)
        .unwrap();

    initialize_vault(&mut env, &client);

    let vault_state_account: VaultState = common::load(&env, &vault_state);
    assert_eq!(vault_state_account.state_bump, state_bump);
    assert_eq!(vault_state_account.vault_bump, vault_bump);
    assert!(env.svm.get_account(&vault).is_none());
}

#[test]
fn initialize_rejects_existing_vault_state() {
    let mut env = Env::new();
    let client = VaultClient::new();
    let (vault_state, state_bump) = VaultState::find_pda(&env.user.pubkey());
    initialize_vault(&mut env, &client);

    let initialize_ix = client.initialize(env.user.pubkey());
    let error_text = send_tx_expect_error(&mut env.svm, initialize_ix, &[&env.user]);

    assert!(error_text.contains("already in use"), "{error_text}");
    let vault_state_account: VaultState = common::load(&env, &vault_state);
    assert_eq!(vault_state_account.state_bump, state_bump);
}
