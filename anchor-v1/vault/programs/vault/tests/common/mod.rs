#![allow(dead_code)]

pub mod client;
mod env;

pub use env::Env;

use {
    anchor_lang::{prelude::Pubkey, solana_program::instruction::Instruction, AccountDeserialize},
    litesvm::LiteSVM,
    solana_keypair::Keypair,
    solana_message::{v0, VersionedMessage},
    solana_signer::Signer,
    solana_transaction::versioned::VersionedTransaction,
    vault::state::vault_state::VaultState,
};

use client::VaultClient;

pub fn find_vault_accounts(user: &Pubkey) -> (Pubkey, Pubkey) {
    let (vault_state, _) = VaultState::find_pda(user);
    let (vault, _) = VaultState::find_vault_pda(&vault_state);
    (vault_state, vault)
}

pub fn initialize_vault(env: &mut Env, client: &VaultClient) {
    let initialize_ix = client.initialize(env.user.pubkey());
    send_tx_expect_ok(&mut env.svm, initialize_ix, &[&env.user]);
}

fn signed_v0_tx(svm: &LiteSVM, ix: Instruction, signers: &[&Keypair]) -> VersionedTransaction {
    let payer = signers[0];

    let message =
        v0::Message::try_compile(&payer.pubkey(), &[ix], &[], svm.latest_blockhash()).unwrap();

    VersionedTransaction::try_new(VersionedMessage::V0(message), signers).unwrap()
}

pub fn load<T: AccountDeserialize>(env: &Env, key: &Pubkey) -> T {
    let account = env.svm.get_account(key).unwrap();
    let mut data: &[u8] = &account.data;
    T::try_deserialize(&mut data).unwrap()
}

pub fn account_lamports(env: &Env, key: &Pubkey) -> u64 {
    match env.svm.get_account(key) {
        Some(account) => account.lamports,
        None => 0,
    }
}

pub fn send_tx_expect_ok(svm: &mut LiteSVM, ix: Instruction, signers: &[&Keypair]) -> u64 {
    let tx = signed_v0_tx(svm, ix, signers);
    let result = svm.send_transaction(tx);
    advance_blockhash(svm);
    match result {
        Ok(meta) => {
            println!("{}", meta.pretty_logs());
            meta.fee
        }
        Err(failed) => {
            println!("Transaction failed: {:?}", failed.err);
            println!("{}", failed.meta.pretty_logs());
            panic!("send_transaction failed: {:?}", failed.err);
        }
    }
}

pub fn send_tx_expect_error(svm: &mut LiteSVM, ix: Instruction, signers: &[&Keypair]) -> String {
    let tx = signed_v0_tx(svm, ix, signers);
    let result = svm.send_transaction(tx);
    advance_blockhash(svm);
    match result {
        Ok(meta) => {
            println!("{}", meta.pretty_logs());
            panic!("expected transaction to fail");
        }
        Err(failed) => {
            println!("Transaction failed as expected: {:?}", failed.err);
            println!("{}", failed.meta.pretty_logs());
            format!("{:#?} {}", failed.err, failed.meta.pretty_logs())
        }
    }
}

// LiteSVM reuses one blockhash until expire_blockhash, so a retry would repeat the signature.
pub fn advance_blockhash(svm: &mut LiteSVM) {
    svm.expire_blockhash();
}
