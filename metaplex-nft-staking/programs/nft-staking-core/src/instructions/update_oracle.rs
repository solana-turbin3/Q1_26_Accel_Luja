use anchor_lang::{
    prelude::*,
    system_program::{transfer, Transfer},
};

use crate::{
    constants::REWARD_IN_LAMPORTS,
    errors::StakingError,
    helper::{is_within_timerange, transfer_allowed},
    state::{ExternalValidationResult, Oracle, OracleValidation},
};

#[derive(Accounts)]
pub struct UpdateOracle<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds=[b"oracle"],
        bump
    )]
    pub oracle: Account<'info, Oracle>,

    /// CHECK: verified by MPL program
    #[account(mut)]
    pub collection: UncheckedAccount<'info>,
    #[account(
        seeds=[b"reward_vault",oracle.key().as_ref()],
        bump
    )]
    pub reward_vault: SystemAccount<'info>,
    pub system_program: Program<'info, System>,
}

impl<'info> UpdateOracle<'info> {
    pub fn update_oracle(&mut self) -> Result<()> {
        let approved = OracleValidation::V1 {
            create: ExternalValidationResult::Pass,
            transfer: ExternalValidationResult::Approved,
            burn: ExternalValidationResult::Pass,
            update: ExternalValidationResult::Pass,
        };
        let rejected = OracleValidation::V1 {
            create: ExternalValidationResult::Pass,
            transfer: ExternalValidationResult::Rejected,
            burn: ExternalValidationResult::Pass,
            update: ExternalValidationResult::Pass,
        };

        match transfer_allowed(Clock::get()?.unix_timestamp) {
            true => {
                require!(
                    self.oracle.validation == rejected,
                    StakingError::OracleAlreadyUpdated
                );
                self.oracle.validation = approved
            }
            false => {
                require!(
                    self.oracle.validation == approved,
                    StakingError::OracleAlreadyUpdated
                );
                self.oracle.validation = rejected
            }
        }

        let vault_value = self.reward_vault.lamports();
        let oracle_key = self.oracle.key();
        let signer_seeds: &[&[u8]] = &[
            b"reward_vault",
            oracle_key.as_ref(),
            &[self.oracle.vault_bump],
        ];

        if is_within_timerange(Clock::get()?.unix_timestamp) && vault_value > REWARD_IN_LAMPORTS {
            transfer(
                CpiContext::new_with_signer(
                    self.system_program.to_account_info(),
                    Transfer {
                        from: self.reward_vault.to_account_info(),
                        to: self.user.to_account_info(),
                    },
                    &[signer_seeds],
                ),
                REWARD_IN_LAMPORTS,
            )?
        }
        Ok(())
    }
}
