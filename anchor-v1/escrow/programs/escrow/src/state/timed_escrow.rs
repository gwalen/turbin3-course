use anchor_lang::prelude::*;

// seeds: [b"timed_escrow", maker, seed.to_le_bytes()]
#[account]
#[derive(InitSpace)]
pub struct TimedEscrow {
    pub seed: u64,
    pub maker: Pubkey,
    pub mint_a: Pubkey,
    pub mint_b: Pubkey,
    pub receive: u64,
    pub expires_at: i64,
    pub bump: u8,
}

impl TimedEscrow {
    pub const LEN: usize = 8 + Self::INIT_SPACE;
    pub const SEED_PREFIX: &'static [u8] = b"timed_escrow";

    pub fn find_pda(maker: &Pubkey, seed: u64) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[Self::SEED_PREFIX, maker.as_ref(), &seed.to_le_bytes()],
            &crate::ID,
        )
    }
}
