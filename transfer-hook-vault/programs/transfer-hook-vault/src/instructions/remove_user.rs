use crate::{constant::USER_SEED, state::User};
use anchor_lang::prelude::*;

#[derive(Accounts)]
#[instruction(user:Pubkey)]
pub struct RemoveUser<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        mut,
        close=admin,
        seeds=[USER_SEED,user.key().as_ref()],
        bump,
    )]
    pub user_info: Account<'info, User>,
    pub system_program: Program<'info, System>,
}

impl<'info> RemoveUser<'info> {
    pub fn remove_user(&mut self, _user: Pubkey) -> Result<()> {
        Ok(())
    }
}
