use {
    litesvm::{types::TransactionMetadata, LiteSVM},
    mpl_core::{accounts::BaseAssetV1, ID as MPL_CORE_ID},
    solana_instruction::Instruction,
    solana_keypair::Keypair,
    solana_message::{Message, VersionedMessage},
    solana_program_pack::Pack,
    solana_pubkey::Pubkey,
    solana_signer::Signer,
    solana_transaction::versioned::VersionedTransaction,
    spl_token_interface::state::Account as TokenAccount,
};

pub const AIRDROP_LAMPORTS: u64 = 10_000_000_000;
pub const DECIMALS: u8 = 6;
pub const MINT_AMOUNT: u64 = 1_000_000_000;
pub const TRANSFER_AMOUNT: u64 = 400_000_000;
pub const NFT_NAME: &str = "Turbin3 Core";
pub const NFT_URI: &str = "https://example.com/turbin3.json";
pub const NFT_NAME_UPDATED: &str = "Turbin3 Core Revised";
pub const NFT_URI_UPDATED: &str = "https://example.com/turbin3-v2.json";
/// Flat fee MPL Core keeps on the asset at creation. Burn does not refund it.
pub const MPL_CORE_CREATE_FEE: u64 = 1_500_000;

pub fn send(
    svm: &mut LiteSVM,
    fee_payer: &Keypair,
    instructions: &[Instruction],
    signers: &[&Keypair],
) -> TransactionMetadata {
    let blockhash = svm.latest_blockhash();
    let message = Message::new_with_blockhash(instructions, Some(&fee_payer.pubkey()), &blockhash);
    let transaction = VersionedTransaction::try_new(VersionedMessage::Legacy(message), signers)
        .expect("transaction should sign");

    svm.send_transaction(transaction)
        .unwrap_or_else(|err| panic!("{}\n{}", err.err, err.meta.pretty_logs()))
}

pub fn token_account(svm: &LiteSVM, address: &Pubkey) -> TokenAccount {
    let account = svm
        .get_account(address)
        .unwrap_or_else(|| panic!("missing token account {address}"));
    TokenAccount::unpack(&account.data).expect("token account data")
}

pub fn load_asset(svm: &LiteSVM, asset: &Pubkey) -> BaseAssetV1 {
    let account = svm
        .get_account(asset)
        .unwrap_or_else(|| panic!("missing asset {asset}"));
    assert_eq!(account.owner, MPL_CORE_ID);
    BaseAssetV1::from_bytes(&account.data).expect("asset account data")
}
