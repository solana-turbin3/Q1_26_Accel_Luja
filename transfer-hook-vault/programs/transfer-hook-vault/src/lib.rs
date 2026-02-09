use anchor_lang::prelude::*;

declare_id!("3LAFnxsE3TqNpM1fnbm7NLFiegMognzYn3ukFu4qP1xe");

#[program]
pub mod transfer_hook_vault {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
