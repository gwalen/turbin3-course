use anchor_lang::prelude::*;

// seeds: [b"state", user]; the vault PDA: [b"vault", vault_state]
#[account]
#[derive(InitSpace)]
pub struct VaultState {
    pub vault_bump: u8,
    pub state_bump: u8,
}

impl VaultState {
    pub const LEN: usize = 8 + Self::INIT_SPACE;
    pub const SEED_PREFIX: &'static [u8] = b"state";
    pub const VAULT_SEED_PREFIX: &'static [u8] = b"vault";
    // 0.01 SOL
    pub const MIN_BALANCE: u64 = 10_000_000;

    pub fn find_pda(user: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(&[Self::SEED_PREFIX, user.as_ref()], &crate::ID)
    }

    pub fn find_vault_pda(vault_state: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(&[Self::VAULT_SEED_PREFIX, vault_state.as_ref()], &crate::ID)
    }

    pub fn create_vault_signer_seeds<'a>(
        vault_state: &'a Pubkey,
        vault_bump: &'a [u8],
    ) -> [&'a [u8]; 3] {
        [Self::VAULT_SEED_PREFIX, vault_state.as_ref(), vault_bump]
    }
}
