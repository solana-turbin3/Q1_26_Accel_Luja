use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Serialize, de::DeserializeOwned};
use std::error::Error;
use wincode::{SchemaRead, SchemaWrite, config::DefaultConfig};

pub trait Serializer<T> {
    fn to_bytes(value: &T) -> Result<Vec<u8>, Box<dyn Error>>;
    fn from_bytes(bytes: &Vec<u8>) -> Result<T, Box<dyn Error>>;
}

pub struct Borsh;
pub struct Wincode;
pub struct Json;

impl<T: BorshSerialize + BorshDeserialize> Serializer<T> for Borsh {
    fn to_bytes(value: &T) -> Result<Vec<u8>, Box<dyn Error>> {
        borsh::to_vec(value).map_err(|e| e.into())
    }
    fn from_bytes(bytes: &Vec<u8>) -> Result<T, Box<dyn Error>> {
        borsh::from_slice(bytes).map_err(|e| e.into())
    }
}

impl<T: Serialize + DeserializeOwned> Serializer<T> for Json {
    fn to_bytes(value: &T) -> Result<Vec<u8>, Box<dyn Error>> {
        serde_json::to_vec(value).map_err(|e| e.into())
    }

    fn from_bytes(bytes: &Vec<u8>) -> Result<T, Box<dyn Error>> {
        serde_json::from_slice(bytes).map_err(|e| e.into())
    }
}

impl<T: SchemaWrite<DefaultConfig, Src = T> + for<'de> SchemaRead<'de, DefaultConfig, Dst = T>>
    Serializer<T> for Wincode
{
    fn to_bytes(value: &T) -> Result<Vec<u8>, Box<dyn Error>> {
        wincode::serialize(value).map_err(|e| e.into())
    }

    fn from_bytes(bytes: &Vec<u8>) -> Result<T, Box<dyn Error>> {
        wincode::deserialize(bytes).map_err(|e| e.into())
    }
}
