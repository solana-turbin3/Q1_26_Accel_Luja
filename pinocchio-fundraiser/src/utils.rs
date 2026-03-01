macro_rules! impl_load {
    ($t:ty) => {
        impl $t {
            pub const LEN: usize = core::mem::size_of::<Self>();

            #[inline(always)]
            pub fn load(data: &[u8]) -> Result<&Self, pinocchio::error::ProgramError> {
                use core::mem;

                if data.len() != Self::LEN {
                    return Err(pinocchio::error::ProgramError::InvalidAccountData);
                }
                let ptr = data.as_ptr();
                if (ptr as usize) % mem::align_of::<Self>() != 0 {
                    return Err(pinocchio::error::ProgramError::InvalidAccountData);
                }

                Ok(unsafe { &*(ptr as *const Self) })
            }

            #[inline(always)]
            pub fn load_mut(data: &mut [u8]) -> Result<&mut Self, pinocchio::error::ProgramError> {
                use core::mem;

                if data.len() != Self::LEN {
                    return Err(pinocchio::error::ProgramError::InvalidAccountData);
                }
                let ptr = data.as_mut_ptr();
                if (ptr as usize) % mem::align_of::<Self>() != 0 {
                    return Err(pinocchio::error::ProgramError::InvalidAccountData);
                }

                Ok(unsafe { &mut *(ptr as *mut Self) })
            }
        }
    };
}

pub(crate) use impl_load;
