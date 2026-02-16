use crate::{constant::VAULT_SEED, state::Vault};
use anchor_lang::{
    prelude::*,
    solana_program::program::invoke,
    system_program::{create_account, CreateAccount},
};
use anchor_spl::{
    token_2022::spl_token_2022::{
        extension::{
            transfer_fee::instruction::initialize_transfer_fee_config,
            transfer_hook::instruction::initialize, ExtensionType,
        },
        instruction::initialize_mint2,
        state::Mint,
    },
    token_interface::TokenInterface,
};

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        init,
        payer = admin,
        space = Vault::LEN,
        seeds = [VAULT_SEED,admin.key().as_ref()],
        bump
    )]
    pub vault: Account<'info, Vault>,

    ///CHECK: Mint passed as acount info to be initialized
    #[account(mut, signer)]
    pub mint: AccountInfo<'info>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

impl<'info> Initialize<'info> {
    pub fn initialize(&mut self, bump: InitializeBumps) -> Result<()> {
        self.vault.set_inner(Vault {
            admin: self.admin.key(),
            mint: self.mint.key(),
            total_amount: 0,
            bump: bump.vault,
        });
        Ok(())
    }

    pub fn init_mint(&mut self, fee: u8, decimal: u8) -> Result<()> {
        let extension_types = vec![
            ExtensionType::TransferHook,
            ExtensionType::TransferFeeConfig,
        ];
        let space = ExtensionType::try_calculate_account_len::<Mint>(&extension_types).unwrap();
        let lamports = Rent::get()?.minimum_balance(space);

        create_account(
            CpiContext::new(
                self.system_program.to_account_info(),
                CreateAccount {
                    from: self.admin.to_account_info(),
                    to: self.mint.to_account_info(),
                },
            ),
            lamports,
            space as u64,
            &self.token_program.key(),
        )?;

        let init_transfer_hook_ix = initialize(
            &self.token_program.key(),
            &self.mint.key(),
            Some(self.vault.key()),
            Some(crate::ID),
        )?;
        invoke(&init_transfer_hook_ix, &[self.mint.to_account_info()])?;

        let inti_transfer_fee_ix = initialize_transfer_fee_config(
            &self.token_program.key(),
            &self.mint.key(),
            Some(&self.vault.key()),
            Some(&self.admin.key()),
            fee.into(),
            decimal.into(),
        )?;
        invoke(&inti_transfer_fee_ix, &[self.mint.to_account_info()])?;

        let init_mint_ix = initialize_mint2(
            &self.token_program.key(),
            &self.mint.key(),
            &self.vault.key(),
            Some(&self.vault.key()),
            decimal,
        )?;
        invoke(&init_mint_ix, &[self.mint.to_account_info()])?;
        Ok(())
    }
}
