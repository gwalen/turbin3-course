use anchor_lang::prelude::*;

pub mod handlers;
pub mod instructions;
pub mod state;
pub mod utils;

pub use instructions::*;

declare_id!("A7Xa1qE7ykhr5wEkSth2DBH8m7LGPiX9PF8AJvmkwQeF");

#[program]
pub mod escrow {
    use super::*;

    pub fn make(ctx: Context<Make>, seed: u64, deposit: u64, receive: u64) -> Result<()> {
        handlers::make::handle(ctx, seed, deposit, receive)
    }

    pub fn take(ctx: Context<Take>) -> Result<()> {
        handlers::take::handle(ctx)
    }

    pub fn refund(ctx: Context<Refund>) -> Result<()> {
        handlers::refund::handle(ctx)
    }

    pub fn make_timed(
        ctx: Context<MakeTimed>,
        seed: u64,
        deposit: u64,
        receive: u64,
        expires_at: i64,
    ) -> Result<()> {
        handlers::make_timed::handle(ctx, seed, deposit, receive, expires_at)
    }

    pub fn take_expired(ctx: Context<TakeExpired>) -> Result<()> {
        handlers::take_expired::handle(ctx)
    }

    pub fn refund_expired(ctx: Context<RefundExpired>) -> Result<()> {
        handlers::refund_expired::handle(ctx)
    }
}
