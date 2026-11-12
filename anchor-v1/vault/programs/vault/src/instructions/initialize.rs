use anchor_lang::prelude::*;

use crate::state::vault_state::VaultState;

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        init,
        payer = user,
        space = VaultState::LEN,
        seeds = [VaultState::SEED_PREFIX, user.key().as_ref()],
        bump
    )]
    pub vault_state: Account<'info, VaultState>,

    #[account(
        seeds = [VaultState::VAULT_SEED_PREFIX, vault_state.key().as_ref()],
        bump
    )]
    pub vault: SystemAccount<'info>,

    pub system_program: Program<'info, System>,
}
