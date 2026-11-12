use anchor_lang::prelude::*;

#[error_code]
pub enum VaultError {
    #[msg("Amount must be greater than zero")]
    ZeroAmount,
    #[msg("Deposit must be at least 0.01 SOL")]
    DepositTooSmall,
    #[msg("Vault balance is lower than the requested amount")]
    InsufficientFunds,
    #[msg("Vault balance would be non-zero but below 0.01 SOL")]
    BelowMinBalance,
}
