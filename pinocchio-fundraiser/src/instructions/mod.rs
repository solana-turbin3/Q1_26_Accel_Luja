use pinocchio::error::ProgramError;

pub mod checker;
pub mod contribute;
pub mod initialize;
pub mod refund;

#[repr(u8)]
#[derive(Debug)]
pub enum FundraiseInstructions {
    Initialize,
    Contribute,
    Checker,
    Refund,
}

impl TryFrom<&u8> for FundraiseInstructions {
    type Error = ProgramError;

    fn try_from(value: &u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(FundraiseInstructions::Initialize),
            1 => Ok(FundraiseInstructions::Contribute),
            2 => Ok(FundraiseInstructions::Checker),
            3 => Ok(FundraiseInstructions::Refund),
            _ => Err(ProgramError::InvalidInstructionData),
        }
    }
}
