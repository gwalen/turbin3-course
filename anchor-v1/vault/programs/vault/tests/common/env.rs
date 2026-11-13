use {
    anchor_lang::prelude::Pubkey, litesvm::LiteSVM, solana_keypair::Keypair, solana_signer::Signer,
};

const USER_AIRDROP_LAMPORTS: u64 = 10_000_000_000;

pub struct Env {
    pub svm: LiteSVM,
    pub program_id: Pubkey,
    pub user: Keypair,
}

impl Env {
    pub fn new() -> Self {
        let program_id = vault::id();
        let user = Keypair::new();

        let mut svm = LiteSVM::new();
        let bytes = include_bytes!(concat!(env!("CARGO_TARGET_TMPDIR"), "/../deploy/vault.so"));
        svm.add_program(program_id, bytes).unwrap();
        svm.airdrop(&user.pubkey(), USER_AIRDROP_LAMPORTS).unwrap();

        Self {
            svm,
            program_id,
            user,
        }
    }
}
