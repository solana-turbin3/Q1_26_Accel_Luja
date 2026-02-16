use crate::{
    constant::{USER_SEED, VAULT_SEED},
    error::TransferError,
    state::{User, Vault},
};
use anchor_lang::prelude::*;
use anchor_lang::system_program::{transfer, Transfer};
use anchor_spl::{
    associated_token::AssociatedToken,
    token_2022::{mint_to_checked, MintToChecked},
    token_interface::{Mint, TokenAccount, TokenInterface},
};

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        mut,
        seeds = [VAULT_SEED,vault.admin.key().as_ref()],
        bump
    )]
    pub vault: Account<'info, Vault>,
    #[account(
        mut,
        seeds=[USER_SEED,user.key().as_ref()],
        bump,
    )]
    pub user_info: Account<'info, User>,

    #[account(
        mut,
        associated_token::mint=mint,
        associated_token::authority=user,
        associated_token::token_program=token_program
    )]
    pub user_token_ata: InterfaceAccount<'info, TokenAccount>,

    #[account(mut)]
    pub mint: InterfaceAccount<'info, Mint>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

impl<'info> Deposit<'info> {
    pub fn deposit(&mut self, amount: u64) -> Result<()> {
        require!(
            self.user.key() == self.user_info.address.key(),
            TransferError::Unauthorized
        );

        transfer(
            CpiContext::new(
                self.system_program.to_account_info(),
                Transfer {
                    from: self.user.to_account_info(),
                    to: self.vault.to_account_info(),
                },
            ),
            amount,
        )?;
        let admin_key = self.vault.admin.key();
        let vault_signer_seeds: &[&[&[u8]]] =
            &[&[VAULT_SEED, admin_key.as_ref(), &[self.vault.bump]]];

        mint_to_checked(
            CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                MintToChecked {
                    mint: self.mint.to_account_info(),
                    to: self.user_token_ata.to_account_info(),
                    authority: self.vault.to_account_info(),
                },
                vault_signer_seeds,
            ),
            amount,
            self.mint.decimals,
        )?;
        Ok(())
    }
}
