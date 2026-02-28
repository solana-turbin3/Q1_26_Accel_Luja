use pinocchio::{
    cpi::{Seed, Signer},
    error::ProgramError,
    AccountView, ProgramResult,
};
use pinocchio_pubkey::derive_address;
use pinocchio_token::{
    instructions::{CloseAccount, Transfer},
    state::TokenAccount,
};

use crate::states::Escrow;

pub fn process_refund_instruction(accounts: &[AccountView], _data: &[u8]) -> ProgramResult {
    let [maker, maker_ata_a, escrow_account, escrow_ata, _remaining_program @ ..] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    let (amount_to_give, bump) = {
        let escrow_state = Escrow::from_account_info(escrow_account)?;

        if escrow_state.maker != *maker.address().as_array() {
            return Err(ProgramError::InvalidAccountData);
        }
        (escrow_state.amount_to_give, escrow_state.bumps)
    };

    {
        let maker_ata_state = TokenAccount::from_account_view(maker_ata_a)?;
        if maker_ata_state.owner() != maker.address() {
            return Err(ProgramError::InvalidAccountData);
        };
    }

    let seed = [b"escrow".as_ref(), maker.address().as_ref(), &[bump]];

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
        from: escrow_ata,
        to: maker_ata_a,
        authority: escrow_account,
        amount: u64::from_le_bytes(amount_to_give),
    }
    .invoke_signed(&[Signer::from(&seeds)])?;

    CloseAccount {
        account: escrow_ata,
        authority: escrow_account,
        destination: maker,
    }
    .invoke_signed(&[Signer::from(&seeds)])?;

    let escrow_lamports = escrow_account.lamports();
    maker.set_lamports(maker.lamports() + escrow_lamports);
    escrow_account.set_lamports(0);

    Ok(())
}
