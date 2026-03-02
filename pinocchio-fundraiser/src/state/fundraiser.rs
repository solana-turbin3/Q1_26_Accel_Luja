use crate::utils::impl_load;
#[repr(C, packed)]
pub struct Fundraiser {
    pub maker: [u8; 32],
    pub mint: [u8; 32],
    pub amount_to_raise: [u8; 8],
    pub current_amount: [u8; 8],
    pub time_started: [u8; 8],
    pub duration: u8,
    pub bump: u8,
}

impl_load!(Fundraiser);
