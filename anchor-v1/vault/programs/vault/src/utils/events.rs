use anchor_lang::prelude::*;

#[event]
pub struct VaultInitialized {
    pub user: Pubkey,
    pub vault_state: Pubkey,
    pub vault: Pubkey,
}

#[event]
pub struct Deposited {
    pub user: Pubkey,
    pub vault: Pubkey,
    pub amount: u64,
    pub vault_balance: u64,
}

#[event]
pub struct Withdrawn {
    pub user: Pubkey,
    pub vault: Pubkey,
    pub amount: u64,
    pub vault_balance: u64,
}

#[event]
pub struct VaultClosed {
    pub user: Pubkey,
    pub vault: Pubkey,
    pub amount: u64,
}
