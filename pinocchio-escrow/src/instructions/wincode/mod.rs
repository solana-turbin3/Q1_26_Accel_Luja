pub mod make2;
pub mod refund2;
pub mod take2;

pub use make2::*;
pub use refund2::*;
pub use take2::*;

use pinocchio::error::ProgramError;

#[repr(u8)]
pub enum EscrowInstructions2 {
    Make2,
    Take2,
    Refund2,
}

impl TryFrom<&u8> for EscrowInstructions2 {
    type Error = ProgramError;

    fn try_from(value: &u8) -> Result<Self, Self::Error> {
        match value {
            4 => Ok(EscrowInstructions2::Make2),
            5 => Ok(EscrowInstructions2::Take2),
            6 => Ok(EscrowInstructions2::Refund2),
            _ => Err(ProgramError::InvalidInstructionData),
        }
    }
}
