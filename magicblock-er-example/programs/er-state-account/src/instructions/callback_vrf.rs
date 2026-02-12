use crate::state::UserAccount;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct CallbackVrf<'info> {
    #[account(address = ephemeral_vrf_sdk::consts::VRF_PROGRAM_IDENTITY)]
    pub vrf_program_identity: Signer<'info>,
    #[account(mut)]
    pub user_account: Account<'info, UserAccount>,
}

impl<'info> CallbackVrf<'info> {
    pub fn process(&mut self, randomness: [u8; 32]) -> Result<()> {
        let rnd_u64 = ephemeral_vrf_sdk::rnd::random_u64(&randomness);
        self.user_account.data = rnd_u64;
        Ok(())
    }
}
