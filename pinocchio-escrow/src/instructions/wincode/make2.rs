use pinocchio::{
    cpi::{Seed, Signer},
    error::ProgramError,
    sysvars::{rent::Rent, Sysvar},
    AccountView, ProgramResult,
};
use pinocchio_associated_token_account::instructions::Create;
use pinocchio_pubkey::derive_address;
use pinocchio_system::instructions::CreateAccount;
use pinocchio_token::instructions::Transfer;
use wincode::SchemaRead;

use crate::states::Escrow;

#[derive(SchemaRead)]
pub struct MakeInstructionData {
    pub amount_to_receive: u64,
    pub amount_to_give: u64,
    pub bump: u8,
}

pub fn process_make2_instruction(accounts: &[AccountView], data: &[u8]) -> ProgramResult {
    let [maker, mint_a, mint_b, escrow_account, maker_ata_a, escrow_ata, system_program, token_program, _associated_token_program @ ..] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    {
        let maker_ata_state = pinocchio_token::state::TokenAccount::from_account_view(maker_ata_a)?;
        if maker_ata_state.owner() != maker.address() {
            return Err(ProgramError::IllegalOwner);
        };
        if maker_ata_state.mint() != mint_a.address() {
            return Err(ProgramError::InvalidAccountData);
        }
    }

    let make_data = wincode::deserialize::<MakeInstructionData>(data)
        .map_err(|_| ProgramError::InvalidAccountData)?;
    let amounts_to_receive = make_data.amount_to_receive;
    let amounts_to_give = make_data.amount_to_give;
    let bump = make_data.bump;

    let seed: [&[u8]; 3] = [b"escrow".as_ref(), maker.address().as_ref(), &[bump]];

    let escrow_account_pda = derive_address(&seed, None, &crate::ID.to_bytes());
    if escrow_account_pda != *escrow_account.address().as_array() {
        return Err(ProgramError::InvalidAccountData);
    }

    let bump_seed = [bump];
    let seed = [
        Seed::from(b"escrow"),
        Seed::from(maker.address().as_array()),
        Seed::from(&bump_seed),
    ];
    let signer_seeds = Signer::from(&seed);

    unsafe {
        if escrow_account.owner() != &crate::ID {
            CreateAccount {
                from: maker,
                to: escrow_account,
                lamports: Rent::get()?.try_minimum_balance(Escrow::LEN)?,
                space: Escrow::LEN as u64,
                owner: &crate::ID,
            }
            .invoke_signed(&[signer_seeds])?;

            let escrow_state = Escrow::from_account_info(escrow_account)?;

            escrow_state.maker = *maker.address().as_array();
            escrow_state.mint_a = *mint_a.address().as_array();
            escrow_state.mint_b = *mint_b.address().as_array();
            escrow_state.amount_to_give = amounts_to_give.to_le_bytes();
            escrow_state.amount_to_receive = amounts_to_receive.to_le_bytes();
            escrow_state.bumps = bump;
        } else {
            return Err(ProgramError::IllegalOwner);
        }
    }

    Create {
        account: escrow_ata,
        funding_account: maker,
        wallet: escrow_account,
        mint: mint_a,
        token_program,
        system_program,
    }
    .invoke()?;
    Transfer {
        authority: maker,
        from: maker_ata_a,
        to: escrow_ata,
        amount: amounts_to_give,
    }
    .invoke()?;
    Ok(())
}
