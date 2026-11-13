use anchor_lang::prelude::{Clock, Pubkey};
use litesvm::LiteSVM;
use solana_keypair::Keypair;
use solana_signer::Signer;

pub const INITIAL_TIMESTAMP: i64 = 1_800_000_000;

pub struct Env {
    pub svm: LiteSVM,
    pub program_id: Pubkey,
    pub maker: Keypair,
    pub taker: Keypair,
    pub other_maker: Keypair,
}

impl Env {
    pub fn new() -> Self {
        let program_id = escrow::ID;
        let maker = Keypair::new_from_array([1; 32]);
        let taker = Keypair::new_from_array([2; 32]);
        let other_maker = Keypair::new_from_array([3; 32]);
        let mut svm = LiteSVM::new();

        let program_bytes =
            include_bytes!(concat!(env!("CARGO_TARGET_TMPDIR"), "/../deploy/escrow.so"));

        svm.add_program(program_id, program_bytes).unwrap();
        for funded_keypair in [&maker, &taker, &other_maker] {
            svm.airdrop(&funded_keypair.pubkey(), 10_000_000_000)
                .unwrap();
        }
        let mut clock: Clock = svm.get_sysvar();
        clock.unix_timestamp = INITIAL_TIMESTAMP;
        svm.set_sysvar(&clock);
        Self {
            svm,
            program_id,
            maker,
            taker,
            other_maker,
        }
    }
}
