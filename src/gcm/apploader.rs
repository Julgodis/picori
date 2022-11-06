use crate::{Ascii, Deserializer, Result};

/// GCM Apploader (`apploader.img`). The boot stages loads the apploader code,
/// The apploader is a small program that loads the main executable and the
/// FST.
#[derive(Debug, Default)]
pub struct Apploader {
    /// Date.
    pub date: String,

    /// Apploader entry point.
    pub entry_point: u32,

    /// Apploader size.
    pub size: u32,

    /// Apploader trailer size.
    pub trailer_size: u32,

    /// Unknown0 (unknown purpose).
    pub unknown: u32,

    /// Apploader data.
    pub data: Vec<u8>,
}

impl Apploader {
    /// Parse GCM Apploader.
    pub fn from_binary<D: Deserializer>(input: &mut D) -> Result<Self> {
        let date = input.deserialize_str::<0x10, Ascii>()?;
        let entry_point = input.deserialize_bu32()?;
        let size = input.deserialize_bu32()?;
        let trailer_size = input.deserialize_bu32()?;
        let unknown = input.deserialize_bu32()?;
        let data_size = (size + trailer_size) as usize;
        let data = input.read_as_vec(data_size)?;

        Ok(Self {
            date,
            entry_point,
            size,
            trailer_size,
            unknown,
            data,
        })
    }
}
