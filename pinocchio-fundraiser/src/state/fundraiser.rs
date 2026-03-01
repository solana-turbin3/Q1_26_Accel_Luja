use wincode::{SchemaRead, SchemaWrite};

#[derive(SchemaRead, SchemaWrite)]
pub struct Fundraiser {
    pub maker: [u8; 32],
    pub mint: [u8; 32],
    pub amount_to_raise: [u8; 8],
    pub current_amount: [u8; 8],
    pub duration: u8,
    pub bump: u8,
}
impl Fundraiser {
    pub const LEN: usize = 32 + 32 + 8 + 8 + 1 + 1;
}

// let data = escrow_account.try_borrow_data()?;
//     let escrow: Escrow = wincode::deserialize(&data)
//         .map_err(|_| ProgramError::InvalidAccountData)?;
