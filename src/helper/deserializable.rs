use std::borrow::Borrow;

use crate::Result;

pub trait DeserializableStringEncoding {
    fn deserialize_str<I>(data: I) -> Result<String>
    where
        I: IntoIterator,
        I::Item: Borrow<u8> + Sized;
}
