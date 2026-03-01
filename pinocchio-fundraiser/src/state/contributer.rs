use wincode::{SchemaRead, SchemaWrite};

#[derive(SchemaRead, SchemaWrite)]
pub struct Contributor {
    pub amount: [u8; 8],
}

impl Contributor {
    pub const LEN: usize = 8;
}
