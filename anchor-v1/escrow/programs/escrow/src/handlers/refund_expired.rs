use crate::{
    instructions::RefundExpired,
    state::timed_escrow::TimedEscrow,
    utils::{
        common::{self, VaultWithdrawal},
        errors::EscrowError,
        events::EscrowRefunded,
    },
};
use anchor_lang::prelude::*;

pub fn handle(ctx: Context<RefundExpired>) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    require!(
        now >= ctx.accounts.escrow.expires_at,
        EscrowError::NotExpired
    );
    let vault_amount = ctx.accounts.vault.amount;
    common::withdraw_and_close_vault(
        VaultWithdrawal {
            token_program: &ctx.accounts.token_program,
            vault: &ctx.accounts.vault,
            mint_a: &ctx.accounts.mint_a,
            destination: ctx.accounts.maker_ata_a.to_account_info(),
            maker: ctx.accounts.maker.to_account_info(),
            escrow: ctx.accounts.escrow.to_account_info(),
        },
        TimedEscrow::SEED_PREFIX,
        ctx.accounts.escrow.seed,
        ctx.accounts.escrow.bump,
    )?;
    emit!(EscrowRefunded {
        escrow: ctx.accounts.escrow.key(),
        maker: ctx.accounts.maker.key(),
        amount: vault_amount,
    });
    Ok(())
}
