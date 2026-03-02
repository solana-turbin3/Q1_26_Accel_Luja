use pinocchio::{
    AccountView, ProgramResult,
    cpi::{Seed, Signer},
    error::ProgramError,
    sysvars::{Sysvar, clock::Clock},
};
use pinocchio_pubkey::derive_address;
use pinocchio_token::{instructions::Transfer, state::TokenAccount};

use crate::{
    constants::SECONDS_TO_DAYS,
    state::{contributer::Contributor, fundraiser::Fundraiser},
    utils::check_ata,
};

pub fn process_refund_instruction(accounts: &[AccountView], _data: &[u8]) -> ProgramResult {
    let [
        maker,
        contributor,
        mint,
        fundraiser_account,
        contributor_ata,
        contributor_account,
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

    let mut fundraiser_data = fundraiser_account.try_borrow_mut()?;
    let mut contributor_data = contributor_account.try_borrow_mut()?;

    let fundraiser_state = Fundraiser::load_mut(&mut fundraiser_data)?;
    let contributor_state = Contributor::load_mut(&mut contributor_data)?;
    let fundraiser_ata_state = TokenAccount::from_account_view(fundraiser_ata)?;

    let bump = fundraiser_state.bump;

    let seed = [b"fundraiser", maker.address().as_ref(), &[bump]];
    let fundraiser_account_pda = derive_address(&seed, None, &crate::ID.to_bytes());

    let amount_to_raise = u64::from_le_bytes(fundraiser_state.amount_to_raise);
    if fundraiser_account_pda != *fundraiser_account.address().as_array() {
        return Err(ProgramError::InvalidAccountData);
    }
    if u64::from_le_bytes(fundraiser_state.current_amount) >= amount_to_raise {
        return Err(ProgramError::InvalidAccountData);
    }

    let current_time = Clock::get()?.unix_timestamp;
    let time_started = i64::from_le_bytes(fundraiser_state.time_started);

    if fundraiser_state.duration > ((current_time - time_started) / SECONDS_TO_DAYS) as u8 {
        return Err(ProgramError::InvalidAccountData);
    }
    if fundraiser_ata_state.amount() > amount_to_raise {
        return Err(ProgramError::InvalidAccountData);
    }

    let bump_seed = [bump];
    let seeds = [
        Seed::from(b"fundraiser"),
        Seed::from(maker.address().as_array()),
        Seed::from(&bump_seed),
    ];

    let transfer_amount = u64::from_le_bytes(contributor_state.amount);

    Transfer {
        amount: transfer_amount,
        from: fundraiser_ata,
        to: contributor_ata,
        authority: fundraiser_account,
    }
    .invoke_signed(&[Signer::from(&seeds)])?;

    fundraiser_state.current_amount = u64::from_le_bytes(fundraiser_state.current_amount)
        .checked_sub(transfer_amount)
        .ok_or(ProgramError::ArithmeticOverflow)?
        .to_le_bytes();

    contributor_state.amount = 0u64.to_le_bytes();

    Ok(())
}
