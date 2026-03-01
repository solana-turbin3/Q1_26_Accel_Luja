use pinocchio::error::ProgramError;
use wincode::{SchemaRead, SchemaWrite};

use crate::utils::impl_load;
#[repr(C)]
#[derive(SchemaRead, SchemaWrite)]
pub struct Contributor {
    pub amount: [u8; 8],
    pub bump: u8,
}

impl_load!(Contributor);
