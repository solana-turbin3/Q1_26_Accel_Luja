use pinocchio::{
    AccountView, ProgramResult,
    cpi::{Seed, Signer},
    error::ProgramError,
};
use pinocchio_pubkey::derive_address;
use pinocchio_token::{instructions::Transfer, state::TokenAccount};

use crate::{state::fundraiser::Fundraiser, utils::check_ata};

pub fn process_checker_instruction(accounts: &[AccountView], _data: &[u8]) -> ProgramResult {
    let [
        maker,
        mint,
        fundraiser_account,
        fundraiser_ata,
        maker_ata,
        _remaining @ ..,
    ] = accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if !maker.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    {
        check_ata!(maker_ata, maker, mint);
        check_ata!(fundraiser_ata, fundraiser_account, mint);
    }

    let mut data = fundraiser_account.try_borrow_mut()?;
    let fundraiser_state = Fundraiser::load_mut(&mut data)?;
    let bump = fundraiser_state.bump;
    let fundraiser_ata_state = TokenAccount::from_account_view(fundraiser_ata)?;
    let seed = [b"fundraiser", maker.address().as_ref(), &[bump]];

    let fundraiser_account_pda = derive_address(&seed, None, &crate::ID.to_bytes());

    if fundraiser_account_pda != *fundraiser_account.address().as_array() {
        return Err(ProgramError::InvalidAccountData);
    }
    if fundraiser_ata_state.amount() < u64::from_le_bytes(fundraiser_state.amount_to_raise) {
        return Err(ProgramError::InvalidAccountData);
    }

    let bump_seed = [bump];
    let seeds = [
        Seed::from(b"fundraiser"),
        Seed::from(maker.address().as_array()),
        Seed::from(&bump_seed),
    ];

    Transfer {
        amount: fundraiser_ata_state.amount(),
        authority: fundraiser_account,
        from: fundraiser_ata,
        to: maker_ata,
    }
    .invoke_signed(&[Signer::from(&seeds)])?;

    Ok(())
}
