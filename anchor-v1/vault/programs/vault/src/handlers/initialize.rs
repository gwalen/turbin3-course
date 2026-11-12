use anchor_lang::prelude::*;

use crate::{
    instructions::initialize::Initialize, state::vault_state::VaultState,
    utils::events::VaultInitialized,
};

pub fn handle(ctx: Context<Initialize>) -> Result<()> {
    ctx.accounts.vault_state.set_inner(VaultState {
        vault_bump: ctx.bumps.vault,
        state_bump: ctx.bumps.vault_state,
    });

    emit!(VaultInitialized {
        user: ctx.accounts.user.key(),
        vault_state: ctx.accounts.vault_state.key(),
        vault: ctx.accounts.vault.key(),
    });

    Ok(())
}
