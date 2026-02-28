#![allow(unexpected_cfgs)]

use pinocchio::{
    address::declare_id, entrypoint, error::ProgramError, AccountView, Address, ProgramResult,
};

use crate::instructions::{
    normal::{process_make_instruction, process_refund_instruction, process_take_instruction},
    wincode::{process_make2_instruction, process_refund2_instruction, process_take2_instruction},
    EscrowInstructions,
};

mod instructions;
mod states;
mod tests;

entrypoint!(process_instruction);

declare_id!("99AW8S9fD1QREzbE25W3uwo7DyjQvuzsYDfcdD6GZbVv");

pub fn process_instruction(
    program_id: &Address,
    accounts: &[AccountView],
    instruction_data: &[u8],
) -> ProgramResult {
    assert_eq!(program_id, &ID);

    let (discriminator, data) = instruction_data
        .split_first()
        .ok_or(ProgramError::InvalidInstructionData)?;

    match EscrowInstructions::try_from(discriminator)? {
        EscrowInstructions::Make => process_make_instruction(accounts, data)?,
        EscrowInstructions::Take => process_take_instruction(accounts, data)?,
        EscrowInstructions::Refund => process_refund_instruction(accounts, data)?,
        EscrowInstructions::Make2 => process_make2_instruction(accounts, data)?,
        EscrowInstructions::Take2 => process_take2_instruction(accounts, data)?,
        EscrowInstructions::Refund2 => process_refund2_instruction(accounts, data)?,
        _ => return Err(ProgramError::InvalidInstructionData)?,
    };

    Ok(())
}
