use pinocchio::{
    cpi::{Seed, Signer},
    error::ProgramError,
    AccountView, ProgramResult,
};
use pinocchio_pubkey::derive_address;
use pinocchio_token::instructions::{CloseAccount, Transfer};

use crate::states::Escrow;

pub fn process_take_instruction(accounts: &[AccountView], data: &[u8]) -> ProgramResult {
    let [maker, taker, escrow_account, maker_ata_b, taker_ata_a, taker_ata_b, escrow_ata, system_program, token_program, _associated_token_program @ ..] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    let (amount_to_give, amount_to_receive, mint_a, mint_b, bump) = {
        let escrow_state = Escrow::from_account_info(escrow_account)?;

        if escrow_state.maker != *maker.address().as_array() {
            return Err(ProgramError::InvalidAccountData);
        }
        (
            escrow_state.amount_to_give,
            escrow_state.amount_to_receive,
            escrow_state.mint_a,
            escrow_state.mint_b,
            escrow_state.bumps,
        )
    };

    let seed = [b"escrow", maker.address().as_ref(), &[bump]];
    let escrow_account_pda = derive_address(&seed, None, &crate::ID.to_bytes());

    if escrow_account_pda != *escrow_account.address().as_array() {
        return Err(ProgramError::InvalidAccountData);
    }
    let bump_seed = [bump];
    let seeds = [
        Seed::from(b"escrow"),
        Seed::from(maker.address().as_array()),
        Seed::from(&bump_seed),
    ];

    Transfer {
        from: taker_ata_b,
        to: maker_ata_b,
        authority: taker,
        amount: u64::from_le_bytes(amount_to_receive),
    }
    .invoke()?;

    Transfer {
        from: escrow_ata,
        to: taker_ata_a,
        authority: escrow_account,
        amount: u64::from_le_bytes(amount_to_give),
    }
    .invoke_signed(&[Signer::from(&seeds)])?;

    CloseAccount {
        account: escrow_account,
        destination: maker,
        authority: escrow_account,
    }
    .invoke_signed(&[Signer::from(&seeds)])?;

    let escrow_lamports = escrow_account.lamports();
    maker.set_lamports(maker.lamports() + escrow_lamports);
    escrow_account.set_lamports(0);

    Ok(())
}
