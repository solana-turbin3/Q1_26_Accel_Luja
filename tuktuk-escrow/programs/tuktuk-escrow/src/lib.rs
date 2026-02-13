use anchor_lang::prelude::*;

declare_id!("6Nn5wecQ6P1PABEd4MrAQAiQz1WN4CTTKcU7DK4VkBs3");

#[program]
pub mod tuktuk_escrow {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
