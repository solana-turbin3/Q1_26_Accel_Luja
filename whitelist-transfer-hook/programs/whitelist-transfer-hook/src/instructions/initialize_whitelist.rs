use crate::state::Whitelist;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct InitializeWhitelist<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        init,
        payer = admin,
        space = 8 + 1, // 8 bytes for discriminator, 1 byte for bump
        seeds = [b"whitelist",admin.key().as_ref()],
        bump
    )]
    pub whitelist: Account<'info, Whitelist>,
    pub system_program: Program<'info, System>,
}

impl<'info> InitializeWhitelist<'info> {
    pub fn initialize_whitelist(&mut self) -> Result<()> {
        Ok(())
    }
}
