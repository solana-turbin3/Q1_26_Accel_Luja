use crate::{
    constants::{AGENT, AGENT_DESC},
    instruction,
    state::Agent,
    ID,
};
use anchor_lang::prelude::*;
use solana_gpt_oracle::ContextAccount;

#[derive(Accounts)]
pub struct Interact<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    ///CHECK: Checked oracle id
    #[account(mut)]
    pub interaction: AccountInfo<'info>,

    #[account(
        seeds=[AGENT,payer.key().as_ref()],
        bump
    )]
    pub agent: Account<'info, Agent>,

    #[account(address=agent.context)]
    pub context_acount: Account<'info, ContextAccount>,

    ///CHECK: Oracle Id
    #[account(address=solana_gpt_oracle::ID)]
    pub oracle_program: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}

impl<'info> Interact<'info> {
    pub fn interact_with_llm(&mut self) -> Result<()> {
        let cpi_program = self.oracle_program.to_account_info();
        let cpi_account = solana_gpt_oracle::cpi::accounts::InteractWithLlm {
            payer: self.payer.to_account_info(),
            context_account: self.context_acount.to_account_info(),
            interaction: self.interaction.to_account_info(),
            system_program: self.system_program.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(cpi_program, cpi_account);
        let desc: [u8; 8] = instruction::CallbackFromLlm::DISCRIMINATOR
            .try_into()
            .expect("Must be 8 bytes");
        solana_gpt_oracle::cpi::interact_with_llm(cpi_ctx, AGENT_DESC.to_string(), ID, desc, None)?;
        Ok(())
    }
}
