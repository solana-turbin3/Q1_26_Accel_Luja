use crate::{constant::USER_SEED, state::User};
use anchor_lang::prelude::*;

#[derive(Accounts)]
#[instruction(user:Pubkey)]
pub struct AddUser<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        init,
        payer=admin,
        seeds=[USER_SEED,user.key().as_ref()],
        space=User::LEN,
        bump
    )]
    pub user_info: Account<'info, User>,
    pub system_program: Program<'info, System>,
}

impl<'info> AddUser<'info> {
    pub fn add_user(&mut self, user: Pubkey, bump: AddUserBumps) -> Result<()> {
        self.user_info.set_inner(User {
            address: user.key(),
            bump: bump.user_info,
        });
        Ok(())
    }
}
