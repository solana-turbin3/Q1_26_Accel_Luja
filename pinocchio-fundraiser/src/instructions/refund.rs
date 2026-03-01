use pinocchio::{
    AccountView, ProgramResult,
    cpi::{Seed, Signer},
    error::ProgramError,
};
use pinocchio_pubkey::derive_address;
use pinocchio_token::{instructions::Transfer, state::TokenAccount};

use crate::state::{contributer::Contributor, fundraiser::Fundraiser};

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

    {
        let contributor_ata_state = TokenAccount::from_account_view(contributor_ata)?;
        if contributor_ata_state.owner() != contributor.address() {
            return Err(ProgramError::InvalidAccountData);
        }
        if contributor_ata_state.mint() != mint.address() {
            return Err(ProgramError::InvalidAccountData);
        }
    }

    let fundraiser_ata_state = TokenAccount::from_account_view(fundraiser_ata)?;
    {
        if fundraiser_ata_state.owner() != fundraiser_account.address() {
            return Err(ProgramError::InvalidAccountData);
        }
        if fundraiser_ata_state.mint() != mint.address() {
            return Err(ProgramError::InvalidAccountData);
        }
    }

    let mut fundraiser_data = fundraiser_account.try_borrow_mut()?;
    let mut contributor_data = contributor_account.try_borrow_mut()?;
    let fundraiser_state = Fundraiser::load_mut(&mut fundraiser_data)?;
    let contributor_state = Contributor::load_mut(&mut contributor_data)?;
    let bump = fundraiser_state.bump;

    let seed = [b"fundraiser", maker.address().as_ref(), &[bump]];
    let fundraiser_account_pda = derive_address(&seed, None, &crate::ID.to_bytes());
    if fundraiser_account_pda != *fundraiser_account.address().as_array() {
        return Err(ProgramError::InvalidAccountData);
    }

    let bump_seed = [bump];
    let seeds = [
        Seed::from(b"fundraiser"),
        Seed::from(maker.address().as_array()),
        Seed::from(&bump_seed),
    ];

    Transfer {
        amount: u64::from_le_bytes(contributor_state.amount),
        from: fundraiser_ata,
        to: contributor_ata,
        authority: fundraiser_account,
    }
    .invoke_signed(&[Signer::from(&seeds)])?;

    fundraiser_state.current_amount = u64::from_le_bytes(fundraiser_state.current_amount)
        .checked_sub(u64::from_le_bytes(contributor_state.amount))
        .ok_or(ProgramError::ArithmeticOverflow)?
        .to_le_bytes();

    Ok(())
}
