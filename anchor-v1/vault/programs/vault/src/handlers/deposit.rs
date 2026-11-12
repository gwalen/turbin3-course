use anchor_lang::{
    prelude::*,
    system_program::{transfer, Transfer},
};

use crate::{
    instructions::deposit::Deposit, state::vault_state::VaultState, utils::errors::VaultError,
    utils::events::Deposited,
};

pub fn handle(ctx: Context<Deposit>, amount: u64) -> Result<()> {
    require!(
        amount >= VaultState::MIN_BALANCE,
        VaultError::DepositTooSmall
    );

    let cpi_accounts = Transfer {
        from: ctx.accounts.user.to_account_info(),
        to: ctx.accounts.vault.to_account_info(),
    };

    let cpi_context = CpiContext::new(ctx.accounts.system_program.key(), cpi_accounts);

    transfer(cpi_context, amount)?;

    emit!(Deposited {
        user: ctx.accounts.user.key(),
        vault: ctx.accounts.vault.key(),
        amount,
        vault_balance: ctx.accounts.vault.lamports(),
    });

    Ok(())
}
