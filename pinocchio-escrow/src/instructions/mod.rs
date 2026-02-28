use pinocchio::error::ProgramError;

pub mod normal;
pub mod wincode;

#[repr(u8)]
pub enum EscrowInstructions {
    Make,
    Take,
    Refund,
    Make2,
    Take2,
    Refund2,
}

impl TryFrom<&u8> for EscrowInstructions {
    type Error = ProgramError;

    fn try_from(value: &u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(EscrowInstructions::Make),
            1 => Ok(EscrowInstructions::Take),
            2 => Ok(EscrowInstructions::Refund),
            3 => Ok(EscrowInstructions::Make2),
            4 => Ok(EscrowInstructions::Take2),
            5 => Ok(EscrowInstructions::Refund2),
            _ => Err(ProgramError::InvalidInstructionData),
        }
    }
}
