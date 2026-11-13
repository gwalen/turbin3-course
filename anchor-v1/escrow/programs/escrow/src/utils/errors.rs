use anchor_lang::prelude::*;

#[error_code]
pub enum EscrowError {
    #[msg("Amount must be greater than zero")]
    ZeroAmount,
    #[msg("Mint A and mint B must differ")]
    SameMint,
    #[msg("Escrow has expired")]
    Expired,
    #[msg("Escrow has not expired yet")]
    NotExpired,
    #[msg("Expiry must be in the future")]
    InvalidExpiry,
    #[msg("Maker must match the escrow maker")]
    UnauthorizedMaker,
    #[msg("Mint A must match the escrow mint A")]
    InvalidMintA,
    #[msg("Mint B must match the escrow mint B")]
    InvalidMintB,
    #[msg("Transfer-fee mints are not supported")]
    TransferFeeMint,
    #[msg("Permanent-delegate mints are not supported")]
    PermanentDelegateMint,
    #[msg("Transfer-hook mints are not supported")]
    TransferHookMint,
    #[msg("Escrow vault must contain tokens before taking an offer")]
    EmptyVault,
}
