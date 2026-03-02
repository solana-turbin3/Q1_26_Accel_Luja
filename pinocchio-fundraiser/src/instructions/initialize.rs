use pinocchio::{
    AccountView, ProgramResult,
    cpi::{Seed, Signer},
    error::ProgramError,
    sysvars::{Sysvar, clock::Clock, rent::Rent},
};
use pinocchio_associated_token_account::instructions::Create;
use pinocchio_pubkey::derive_address;
use pinocchio_system::instructions::CreateAccount;

use crate::{constants::MIN_AMOUNT_TO_RAISE, state::fundraiser::Fundraiser, utils::impl_load};

#[repr(C)]
pub struct InitializeData {
    amount_to_raise: u64,
    duration: u8,
    bump: u8,
}

impl_load!(InitializeData);

pub fn process_initialize_instruction(accounts: &[AccountView], data: &[u8]) -> ProgramResult {
    let [
        maker,
        mint,
        fundraiser_account,
        fundraiser_ata,
        token_program,
        system_program,
        _remaining @ ..,
    ] = accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    if !maker.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let make_data = InitializeData::load(data)?;

    let amount_to_raise = make_data.amount_to_raise;
    let duration = make_data.duration;
    let bump = make_data.bump;

    if amount_to_raise < MIN_AMOUNT_TO_RAISE {
        return Err(ProgramError::InvalidInstructionData);
    }

    let seed = [b"fundraiser", maker.address().as_ref(), &[bump]];
    let fundraiser_account_pda = derive_address(&seed, None, crate::ID.as_array());
    if fundraiser_account_pda != *fundraiser_account.address().as_array() {
        return Err(ProgramError::InvalidAccountData);
    }

    let bump_seed = [bump];
    let seed = [
        Seed::from(b"fundraiser"),
        Seed::from(maker.address().as_array()),
        Seed::from(&bump_seed),
    ];

    unsafe {
        if fundraiser_account.owner() != &crate::ID {
            CreateAccount {
                from: maker,
                to: fundraiser_account,
                lamports: Rent::get()?.try_minimum_balance(Fundraiser::LEN)?,
                space: Fundraiser::LEN as u64,
                owner: &crate::ID,
            }
            .invoke_signed(&[Signer::from(&seed)])?;
            Create {
                account: fundraiser_ata,
                funding_account: maker,
                wallet: fundraiser_account,
                mint,
                token_program,
                system_program,
            }
            .invoke()?;

            let mut data = fundraiser_account.try_borrow_mut()?;
            let fundraiser_state = Fundraiser::load_mut(&mut data)?;
            fundraiser_state.maker = *maker.address().as_array();
            fundraiser_state.mint = *mint.address().as_array();
            fundraiser_state.amount_to_raise = amount_to_raise.to_le_bytes();
            fundraiser_state.current_amount = 0u64.to_le_bytes();
            fundraiser_state.duration = duration;
            fundraiser_state.time_started = Clock::get()?.unix_timestamp.to_le_bytes();
            fundraiser_state.bump = bump;
        } else {
            return Err(ProgramError::IllegalOwner);
        }
    }
    Ok(())
}
