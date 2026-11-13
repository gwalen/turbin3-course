mod common;

use {
    anchor_lang::{
        prelude::Pubkey,
        solana_program::{instruction::Instruction, system_program},
        InstructionData, ToAccountMetas,
    },
    common::{
        account_lamports, client::VaultClient, find_vault_accounts, initialize_vault,
        send_tx_expect_error, send_tx_expect_ok, Env,
    },
    solana_keypair::Keypair,
    solana_signer::Signer,
    vault::{accounts, instruction},
};

fn withdraw_other_vault(user: Pubkey, vault_owner: Pubkey, amount: u64) -> Instruction {
    let (vault_state, vault) = find_vault_accounts(&vault_owner);
    Instruction {
        program_id: vault::ID,
        accounts: accounts::Withdraw {
            user,
            vault,
            vault_state,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
        data: instruction::Withdraw { amount }.data(),
    }
}

fn close_other_vault(user: Pubkey, vault_owner: Pubkey) -> Instruction {
    let (vault_state, vault) = find_vault_accounts(&vault_owner);
    Instruction {
        program_id: vault::ID,
        accounts: accounts::Close {
            user,
            vault,
            vault_state,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
        data: instruction::Close {}.data(),
    }
}

#[test]
fn other_user_cannot_withdraw_or_close() {
    let mut env = Env::new();
    let client = VaultClient::new();
    initialize_vault(&mut env, &client);

    let deposit_lamports = 1_000_000_000;
    let deposit_ix = client.deposit(env.user.pubkey(), deposit_lamports);
    send_tx_expect_ok(&mut env.svm, deposit_ix, &[&env.user]);

    let other_user = Keypair::new();
    env.svm
        .airdrop(&other_user.pubkey(), 1_000_000_000)
        .unwrap();

    let (vault_state, vault) = find_vault_accounts(&env.user.pubkey());
    let vault_balance_before = account_lamports(&env, &vault);
    let owner_balance_before = account_lamports(&env, &env.user.pubkey());
    let vault_state_lamports_before = account_lamports(&env, &vault_state);

    let withdraw_ix =
        withdraw_other_vault(other_user.pubkey(), env.user.pubkey(), deposit_lamports);
    let withdraw_error = send_tx_expect_error(&mut env.svm, withdraw_ix, &[&other_user]);

    assert!(
        withdraw_error.contains("Error Code: ConstraintSeeds"),
        "{withdraw_error}"
    );
    assert_eq!(account_lamports(&env, &vault), vault_balance_before);
    assert_eq!(
        account_lamports(&env, &env.user.pubkey()),
        owner_balance_before
    );
    assert_eq!(
        account_lamports(&env, &vault_state),
        vault_state_lamports_before
    );

    let close_ix = close_other_vault(other_user.pubkey(), env.user.pubkey());
    let close_error = send_tx_expect_error(&mut env.svm, close_ix, &[&other_user]);

    assert!(
        close_error.contains("Error Code: ConstraintSeeds"),
        "{close_error}"
    );
    assert_eq!(account_lamports(&env, &vault), vault_balance_before);
    assert_eq!(
        account_lamports(&env, &env.user.pubkey()),
        owner_balance_before
    );
    assert_eq!(
        account_lamports(&env, &vault_state),
        vault_state_lamports_before
    );
}
