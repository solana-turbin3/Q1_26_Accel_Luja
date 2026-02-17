use anchor_lang::prelude::*;

declare_id!("daTyZ6QYyTP5QpuM3xG3MtF6XLmYP5GQdnJ6mv1fgdB");

#[program]
pub mod tuktuk_gpt_oracle {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
