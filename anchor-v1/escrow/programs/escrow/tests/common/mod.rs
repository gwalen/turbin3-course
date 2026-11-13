#![allow(dead_code)]

pub mod client;
pub mod env;
pub mod token;

pub use env::Env;

use anchor_lang::{prelude::Pubkey, solana_program::instruction::Instruction, AccountDeserialize};
use litesvm::LiteSVM;
use solana_keypair::Keypair;
use solana_message::{v0, VersionedMessage};
use solana_signer::Signer;
use solana_transaction::versioned::VersionedTransaction;

pub const SEED: u64 = 42;
pub const DEPOSIT: u64 = 250_000;
pub const RECEIVE: u64 = 400_000;

pub fn send_tx_expect_ok(svm: &mut LiteSVM, ix: Instruction, signers: &[&Keypair]) -> u64 {
    let tx = sign_tx(svm, ix, signers);
    let transaction_result = svm.send_transaction(tx);
    advance_blockhash(svm);
    match transaction_result {
        Ok(transaction_metadata) => {
            println!("{}", transaction_metadata.pretty_logs());
            transaction_metadata.fee
        }
        Err(transaction_error) => panic!(
            "{:?}\n{}",
            transaction_error.err,
            transaction_error.meta.pretty_logs()
        ),
    }
}

pub fn send_tx_expect_error(svm: &mut LiteSVM, ix: Instruction, signers: &[&Keypair]) -> String {
    let tx = sign_tx(svm, ix, signers);
    let transaction_result = svm.send_transaction(tx);
    advance_blockhash(svm);
    match transaction_result {
        Ok(transaction_metadata) => {
            panic!("Expected failure: {}", transaction_metadata.pretty_logs())
        }
        Err(transaction_error) => {
            let error_text = format!(
                "{:?}\n{}",
                transaction_error.err,
                transaction_error.meta.pretty_logs()
            );
            println!("{error_text}");
            error_text
        }
    }
}

pub fn advance_blockhash(svm: &mut LiteSVM) {
    svm.expire_blockhash();
}

pub fn load<T: AccountDeserialize>(env: &Env, address: &Pubkey) -> T {
    let account = env.svm.get_account(address).unwrap();
    T::try_deserialize(&mut account.data.as_slice()).unwrap()
}

fn sign_tx(svm: &LiteSVM, ix: Instruction, signers: &[&Keypair]) -> VersionedTransaction {
    let tx_message =
        v0::Message::try_compile(&signers[0].pubkey(), &[ix], &[], svm.latest_blockhash()).unwrap();
    VersionedTransaction::try_new(VersionedMessage::V0(tx_message), signers).unwrap()
}
