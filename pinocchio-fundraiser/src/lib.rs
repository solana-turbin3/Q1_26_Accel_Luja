#![allow(unexpected_cfgs)]
use pinocchio::{
    AccountView, Address, ProgramResult, address::declare_id, entrypoint, error::ProgramError,
};

use crate::instructions::{
    FundraiseInstructions, checker::process_checker_instruction,
    contribute::process_contribute_instruction, initialize::process_initialize_instruction,
    refund::process_refund_instruction,
};

mod constants;
mod error;
mod instructions;
mod state;
#[cfg(test)]
mod tests;

declare_id!("9MBbGAcz2KvydYUGVCRNr8ZhiWPpSFXNhE4p8mw61egr");

entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Address,
    accounts: &[AccountView],
    instruction_data: &[u8],
) -> ProgramResult {
    assert_eq!(program_id, &ID);

    let (discriminator, data) = instruction_data
        .split_first()
        .ok_or(ProgramError::InvalidInstructionData)?;

    match FundraiseInstructions::try_from(discriminator)? {
        FundraiseInstructions::Initialize => process_initialize_instruction(accounts, data)?,
        FundraiseInstructions::Contribute => process_contribute_instruction(accounts, data)?,
        FundraiseInstructions::Checker => process_checker_instruction(accounts, data)?,
        FundraiseInstructions::Refund => process_refund_instruction(accounts, data)?,
    }
    Ok(())
}
