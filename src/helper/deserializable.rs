use std::borrow::Borrow;

use super::Deserializer;
use crate::PicoriError;

// TODO: remove?
pub trait Deserializable: Sized {
    fn deserialize<D: Deserializer>(deserializer: &mut D) -> Result<Self, PicoriError>;

    fn deserialize_vector<D: Deserializer>(
        deserializer: &mut D,
        length: usize,
    ) -> Result<Vec<Self>, PicoriError> {
        let mut vec = Vec::with_capacity(length);
        for _ in 0..length {
            vec.push(Self::deserialize(deserializer)?);
        }
        Ok(vec)
    }
}

// TODO: remove?
impl Deserializable for u8 {
    fn deserialize<D: Deserializer>(deserializer: &mut D) -> Result<Self, PicoriError> {
        deserializer.deserialize_u8()
    }
}

pub trait DeserializableStringEncoding {
    fn deserialize_str<I>(data: I) -> Result<String, PicoriError>
    where
        I: IntoIterator,
        I::Item: Borrow<u8> + Sized;
}
