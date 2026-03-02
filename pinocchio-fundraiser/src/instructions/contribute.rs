use pinocchio::{
    AccountView, ProgramResult,
    cpi::{Seed, Signer},
    error::ProgramError,
    sysvars::{Sysvar, clock::Clock, rent::Rent},
};
use pinocchio_pubkey::derive_address;
use pinocchio_system::instructions::CreateAccount;
use pinocchio_token::{instructions::Transfer, state::TokenAccount};

use crate::{
    constants::{MAX_CONTRIBUTION_PERCENTAGE, PERCENTAGE_SCALER, SECONDS_TO_DAYS},
    state::{contributer::Contributor, fundraiser::Fundraiser},
    utils::{check_ata, impl_load},
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

    if !contributor.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    {
        check_ata!(contributor_ata, contributor, mint);
        check_ata!(fundraiser_ata, fundraiser_account, mint);
    }

    let contribute_data = ContributeData::load(data)?;
    let amount = contribute_data.amount;
    let current_time = Clock::get()?.unix_timestamp;

    let bump = contribute_data.bump;
    let contributor_seed = [
        b"contributor",
        fundraiser_account.address().as_ref(),
        contributor.address().as_ref(),
        &[bump],
    ];

    let mut fundraiser_data = fundraiser_account.try_borrow_mut()?;
    let mut contributor_data = contributor_account.try_borrow_mut()?;

    let fundraiser_state = Fundraiser::load_mut(&mut fundraiser_data)?;
    let contributor_state = Contributor::load_mut(&mut contributor_data)?;

    let fundraiser_seed = [
        b"fundraiser",
        fundraiser_state.maker.as_ref(),
        &[fundraiser_state.bump],
    ];

    let fundraiser_account_pda = derive_address(&fundraiser_seed, None, crate::ID.as_array());
    let contributor_account_pda = derive_address(&contributor_seed, None, crate::ID.as_array());

    if fundraiser_account_pda != *fundraiser_account.address().as_array()
        || contributor_account_pda != *contributor_account.address().as_array()
        || fundraiser_state.mint != *mint.address().as_array()
    {
        return Err(ProgramError::InvalidAccountData);
    }

    let max_amount = (u64::from_le_bytes(fundraiser_state.amount_to_raise)
        * MAX_CONTRIBUTION_PERCENTAGE)
        / PERCENTAGE_SCALER;
    let time_started = i64::from_le_bytes(fundraiser_state.time_started);
    let contributor_amount = u64::from_le_bytes(contributor_state.amount);

    if amount == 0
        || amount >= max_amount
        || contributor_amount >= max_amount && (contributor_amount + amount) > max_amount
    {
        return Err(ProgramError::InvalidInstructionData);
    }
    if fundraiser_state.duration > ((current_time - time_started) / SECONDS_TO_DAYS) as u8 {
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
