use pinocchio::{
    AccountView, ProgramResult,
    cpi::{Seed, Signer},
    error::ProgramError,
    sysvars::{Sysvar, clock::Clock},
};
use pinocchio_log::log;
use pinocchio_pubkey::derive_address;
use pinocchio_token::{instructions::Transfer, state::TokenAccount};

use crate::{
    constants::SECONDS_TO_DAYS,
    state::{contributer::Contributor, fundraiser::Fundraiser},
    utils::check_ata,
};

pub fn process_refund_instruction(accounts: &[AccountView], _data: &[u8]) -> ProgramResult {
    let [
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
    log!("----------ATA");

    {
        check_ata!(contributor_ata, contributor, mint);
        check_ata!(fundraiser_ata, fundraiser_account, mint);
    }
    log!("----------ATA");
    log!("ATA");
    let (maker, bump, transfer_amount, amount_to_raise, current_amount, time_started, duration) = {
        let fundraiser_data = fundraiser_account.try_borrow()?;
        let contributor_data = contributor_account.try_borrow()?;

        let fundraiser_state = Fundraiser::load(&fundraiser_data)?;
        let contributor_state = Contributor::load(&contributor_data)?;

        let amount = u64::from_le_bytes(contributor_state.amount);

        (
            fundraiser_state.maker,
            fundraiser_state.bump,
            amount,
            fundraiser_state.amount_to_raise,
            fundraiser_state.current_amount,
            fundraiser_state.time_started,
            fundraiser_state.duration,
        )
    };
    let seed = [b"fundraiser", maker.as_ref(), &[bump]];
    let fundraiser_account_pda = derive_address(&seed, None, &crate::ID.to_bytes());

    let amount_to_raise = u64::from_le_bytes(amount_to_raise);
    if fundraiser_account_pda != *fundraiser_account.address().as_array() {
        return Err(ProgramError::InvalidAccountData);
    }
    if u64::from_le_bytes(current_amount) > amount_to_raise {
        return Err(ProgramError::InvalidAccountData);
    }

    let current_time = Clock::get()?.unix_timestamp;
    let time_started = i64::from_le_bytes(time_started);

    if duration > ((current_time - time_started) / SECONDS_TO_DAYS) as u8 {
        return Err(ProgramError::InvalidAccountData);
    }
    // if fundraiser_ata_state.amount() > amount_to_raise {
    //     return Err(ProgramError::InvalidAccountData);
    // }

    let bump_seed = [bump];
    let seeds = [
        Seed::from(b"fundraiser"),
        Seed::from(maker.as_ref()),
        Seed::from(&bump_seed),
    ];

    Transfer {
        amount: transfer_amount,
        from: fundraiser_ata,
        to: contributor_ata,
        authority: fundraiser_account,
    }
    .invoke_signed(&[Signer::from(&seeds)])?;
    let mut fundraiser_data = fundraiser_account.try_borrow_mut()?;
    let mut contributor_data = contributor_account.try_borrow_mut()?;

    let fundraiser_state = Fundraiser::load_mut(&mut fundraiser_data)?;
    let contributor_state = Contributor::load_mut(&mut contributor_data)?;

    fundraiser_state.current_amount = u64::from_le_bytes(fundraiser_state.current_amount)
        .checked_sub(transfer_amount)
        .ok_or(ProgramError::ArithmeticOverflow)?
        .to_le_bytes();

    contributor_state.amount = 0u64.to_le_bytes();

    Ok(())
}
