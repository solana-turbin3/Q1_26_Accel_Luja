use anchor_lang::prelude::*;

use crate::{
    helper::transfer_allowed,
    state::{ExternalValidationResult, Oracle, OracleValidation},
};

#[derive(Accounts)]
pub struct InitOracle<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        init,
        payer=admin,
        seeds=[b"oracle"],
        space=Oracle::INIT_SPACE,
        bump
    )]
    pub oracle: Account<'info, Oracle>,

    #[account(mut)]
    /// CHECK: verified bby MPL program
    pub collection: UncheckedAccount<'info>,
    #[account(
        seeds=[b"reward_vault",oracle.key().as_ref()],
        bump
    )]
    pub reward_vault: SystemAccount<'info>,
    pub system_program: Program<'info, System>,
}

impl<'info> InitOracle<'info> {
    pub fn init_oracle(&mut self, bumps: InitOracleBumps) -> Result<()> {
        match transfer_allowed(Clock::get()?.unix_timestamp) {
            true => {
                self.oracle.set_inner(Oracle {
                    validation: OracleValidation::V1 {
                        create: ExternalValidationResult::Pass,
                        transfer: ExternalValidationResult::Approved,
                        burn: ExternalValidationResult::Pass,
                        update: ExternalValidationResult::Pass,
                    },
                    bump: bumps.oracle,
                    vault_bump: bumps.reward_vault,
                });
            }
            false => {
                self.oracle.set_inner(Oracle {
                    validation: OracleValidation::V1 {
                        create: ExternalValidationResult::Pass,
                        transfer: ExternalValidationResult::Rejected,
                        burn: ExternalValidationResult::Pass,
                        update: ExternalValidationResult::Pass,
                    },
                    bump: bumps.oracle,
                    vault_bump: bumps.reward_vault,
                });
            }
        }
        Ok(())
    }
}
