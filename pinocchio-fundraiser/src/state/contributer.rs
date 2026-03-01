use crate::utils::impl_load;
#[repr(C)]
pub struct Contributor {
    pub amount: [u8; 8],
    pub bump: u8,
}

impl_load!(Contributor);
