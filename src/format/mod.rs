//! Encoding and decoding of various Nintendo file formats used by GameCube and Wii games.
//! 
//! 

use anyhow::{Result};

pub mod dol;
pub mod gcm;
pub mod ciso;

pub trait Encodable {
    fn to_bytes(&self) -> Result<Vec<u8>>;
}

pub trait Decodable<T> {
    fn identify_as(input: &[u8]) -> bool;
    fn from_bytes(input: &[u8]) -> Result<T>;
}
