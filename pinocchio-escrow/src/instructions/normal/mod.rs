pub mod make;
pub mod refund;
pub mod take;

pub use make::*;
use pinocchio::error::ProgramError;
pub use refund::*;
pub use take::*;

#[repr(u8)]
pub enum EscrowInstructions {
    Make,
    Take,
    Refund,
}

impl TryFrom<&u8> for EscrowInstructions {
    type Error = ProgramError;

    fn try_from(value: &u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(EscrowInstructions::Make),
            1 => Ok(EscrowInstructions::Take),
            2 => Ok(EscrowInstructions::Refund),
            _ => Err(ProgramError::InvalidInstructionData),
        }
    }
}
