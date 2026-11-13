use crate::{
    instructions::MakeTimed,
    state::timed_escrow::TimedEscrow,
    utils::{errors::EscrowError, events::TimedEscrowMade, mint},
};
use anchor_lang::prelude::*;
use anchor_spl::token_interface::{self, TransferChecked};

pub fn handle(
    ctx: Context<MakeTimed>,
    seed: u64,
    deposit: u64,
    receive: u64,
    expires_at: i64,
) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    require!(expires_at > now, EscrowError::InvalidExpiry);
    require!(deposit > 0, EscrowError::ZeroAmount);
    require!(receive > 0, EscrowError::ZeroAmount);
    mint::check_mint_is_supported(&ctx.accounts.mint_a)?;
    mint::check_mint_is_supported(&ctx.accounts.mint_b)?;
    ctx.accounts.escrow.set_inner(TimedEscrow {
        seed,
        maker: ctx.accounts.maker.key(),
        mint_a: ctx.accounts.mint_a.key(),
        mint_b: ctx.accounts.mint_b.key(),
        receive,
        expires_at,
        bump: ctx.bumps.escrow,
    });
    token_interface::transfer_checked(
        CpiContext::new(
            ctx.accounts.token_program_a.key(),
            TransferChecked {
                from: ctx.accounts.maker_ata_a.to_account_info(),
                mint: ctx.accounts.mint_a.to_account_info(),
                to: ctx.accounts.vault.to_account_info(),
                authority: ctx.accounts.maker.to_account_info(),
            },
        ),
        deposit,
        ctx.accounts.mint_a.decimals,
    )?;
    emit!(TimedEscrowMade {
        escrow: ctx.accounts.escrow.key(),
        maker: ctx.accounts.maker.key(),
        mint_a: ctx.accounts.mint_a.key(),
        mint_b: ctx.accounts.mint_b.key(),
        deposit,
        receive,
        expires_at,
    });
    Ok(())
}
