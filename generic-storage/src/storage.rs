use crate::serialize::Serializer;
use std::{error::Error, marker::PhantomData};

pub struct Storage<T, S> {
    pub data: Option<Vec<u8>>,
    pub serializer: S,
    pub _phantom: PhantomData<T>,
}

impl<T, S: Serializer<T>> Storage<T, S> {
    pub fn new(serialier: S) -> Self {
        Storage {
            data: None,
            serializer: serialier,
            _phantom: PhantomData,
        }
    }
    pub fn save(&mut self, value: &T) -> Result<(), Box<dyn Error>> {
        let bytes = S::to_bytes(value)?;
        self.data = Some(bytes);
        Ok(())
    }
    pub fn load(&self) -> Result<T, Box<dyn Error>> {
        let bytes = self.data.as_ref().ok_or("There is no data")?;
        let value = S::from_bytes(bytes)?;
        Ok(value)
    }
    pub fn has_data(&self) -> Result<bool, Box<dyn Error>> {
        Ok(self.data.is_some())
    }
}
