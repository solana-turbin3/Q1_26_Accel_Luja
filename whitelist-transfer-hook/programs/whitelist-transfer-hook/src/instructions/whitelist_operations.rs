use crate::state::whitelist::Whitelist;
use anchor_lang::prelude::*;

#[derive(Accounts)]
#[instruction(user:Pubkey)]
pub struct AddToWhitelist<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        mut,
        seeds = [b"whitelist",user.as_ref()],
        bump,
    )]
    pub whitelist: Account<'info, Whitelist>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(user:Pubkey)]
pub struct RemoveFromWhitelist<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        mut,
        seeds = [b"whitelist",user.as_ref()],
        close=admin,
        bump,
    )]
    pub whitelist: Account<'info, Whitelist>,
    pub system_program: Program<'info, System>,
}

impl<'info> AddToWhitelist<'info> {
    pub fn add_to_whitelist(&mut self, bumps: &AddToWhitelistBumps, _user: Pubkey) -> Result<()> {
        self.whitelist.bump = bumps.whitelist;
        Ok(())
    }
}

impl<'info> RemoveFromWhitelist<'info> {
    pub fn remove_from_whitelist(&mut self, _user: Pubkey) -> Result<()> {
        Ok(())
    }
}
