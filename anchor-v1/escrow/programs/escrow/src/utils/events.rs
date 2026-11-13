use anchor_lang::prelude::*;

#[event]
pub struct EscrowMade {
    pub escrow: Pubkey,
    pub maker: Pubkey,
    pub mint_a: Pubkey,
    pub mint_b: Pubkey,
    pub deposit: u64,
    pub receive: u64,
}

#[event]
pub struct EscrowTaken {
    pub escrow: Pubkey,
    pub maker: Pubkey,
    pub taker: Pubkey,
    pub deposit: u64,
    pub receive: u64,
}

#[event]
pub struct EscrowRefunded {
    pub escrow: Pubkey,
    pub maker: Pubkey,
    pub amount: u64,
}

#[event]
pub struct TimedEscrowMade {
    pub escrow: Pubkey,
    pub maker: Pubkey,
    pub mint_a: Pubkey,
    pub mint_b: Pubkey,
    pub deposit: u64,
    pub receive: u64,
    pub expires_at: i64,
}
