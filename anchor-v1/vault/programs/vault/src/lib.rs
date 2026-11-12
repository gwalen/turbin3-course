use anchor_lang::prelude::*;

pub mod handlers;
pub mod instructions;
pub mod state;
pub mod utils;

pub use instructions::close::*;
pub use instructions::deposit::*;
pub use instructions::initialize::*;
pub use instructions::withdraw::*;
pub use utils::errors;
pub use utils::events;

declare_id!("5N1ZxNpBuLqwcuV8YQ8kS58wav56otLnzHuaTbiHcwyM");

#[program]
pub mod vault {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        handlers::initialize::handle(ctx)
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        handlers::deposit::handle(ctx, amount)
    }

    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        handlers::withdraw::handle(ctx, amount)
    }

    pub fn close(ctx: Context<Close>) -> Result<()> {
        handlers::close::handle(ctx)
    }
}
