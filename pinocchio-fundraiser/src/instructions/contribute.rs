use pinocchio::{
    AccountView, ProgramResult,
    cpi::{Seed, Signer},
    error::ProgramError,
    sysvars::{Sysvar, rent::Rent},
};
use pinocchio_pubkey::derive_address;
use pinocchio_system::instructions::CreateAccount;
use pinocchio_token::{instructions::Transfer, state::TokenAccount};

use crate::{
    state::{contributer::Contributor, fundraiser::Fundraiser},
    utils::impl_load,
};

#[repr(C)]
pub struct ContributeData {
    pub amount: u64,
    pub bump: u8,
}
impl_load!(ContributeData);

pub fn process_contribute_instruction(accounts: &[AccountView], data: &[u8]) -> ProgramResult {
    let [
        contributor,
        mint,
        fundraiser_account,
        contributor_account,
        contributor_ata,
        fundraiser_ata,
        _remaining @ ..,
    ] = accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    {
        let contributor_ata_state = TokenAccount::from_account_view(contributor_ata)?;
        if contributor_ata_state.owner() != contributor.address() {
            return Err(ProgramError::InvalidAccountData);
        }
        if contributor_ata_state.mint() != mint.address() {
            return Err(ProgramError::InvalidAccountData);
        }
    }

    let contribute_data = ContributeData::load(data)?;
    let amount = contribute_data.amount;
    let bump = contribute_data.bump;

    let seed = [
        b"contributor",
        fundraiser_account.address().as_ref(),
        contributor.address().as_ref(),
        &[bump],
    ];

    let contributor_account_pda = derive_address(&seed, None, crate::ID.as_array());
    if contributor_account_pda != *contributor_account.address().as_array() {
        return Err(ProgramError::InvalidAccountData);
    }

    let bump_seed = [bump];
    let seeds = [
        Seed::from(b"contributor"),
        Seed::from(fundraiser_account.address().as_ref()),
        Seed::from(contributor.address().as_ref()),
        Seed::from(&bump_seed),
    ];

    unsafe {
        if contributor_account.owner() != &crate::ID {
            CreateAccount {
                from: contributor,
                to: contributor_account,
                lamports: Rent::get()?.try_minimum_balance(Contributor::LEN)?,
                space: Contributor::LEN as u64,
                owner: &crate::ID,
            }
            .invoke_signed(&[Signer::from(&seeds)])?;
        }

        Transfer {
            from: contributor_ata,
            to: fundraiser_ata,
            authority: contributor,
            amount,
        }
        .invoke()?;

        let mut contributor_data = contributor_account.try_borrow_mut()?;
        let contribute_state = Contributor::load_mut(&mut contributor_data)?;
        contribute_state.amount = u64::from_le_bytes(contribute_state.amount)
            .checked_add(amount)
            .ok_or(ProgramError::ArithmeticOverflow)?
            .to_le_bytes();

        let mut fundraiser_data = fundraiser_account.try_borrow_mut()?;
        let fundraiser_state = Fundraiser::load_mut(&mut fundraiser_data)?;
        fundraiser_state.current_amount = u64::from_le_bytes(fundraiser_state.current_amount)
            .checked_add(amount)
            .ok_or(ProgramError::ArithmeticOverflow)?
            .to_le_bytes();
    }

    Ok(())
}
