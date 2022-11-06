//! [GCM][`crate::gcm`] executable file. This is the [DOL][`crate::dol`] file
//! that contains the actual game code.

use crate::error::ParseProblem;
use crate::helper::{Parser, ProblemLocation, Seeker};
use crate::Result;

/// [GCM][`crate::gcm`] executable file object.
#[derive(Debug, Default)]
pub struct Executable {
    data: Vec<u8>,
}

impl Executable {
    /// Parse [GCM][`crate::gcm`] executable ([DOL][`crate::dol`]) from binary.
    pub fn from_binary<D: Parser + Seeker>(input: &mut D) -> Result<Self> {
        let base = input.position()?;
        let text_offsets = input.bu32_array::<7>()?;
        let data_offsets = input.bu32_array::<11>()?;
        let _ = input.bu32_array::<7>()?;
        let _ = input.bu32_array::<11>()?;
        let text_sizes = input.bu32_array::<7>()?;
        let data_sizes = input.bu32_array::<11>()?;

        let text_iter = text_offsets
            .iter()
            .zip(text_sizes.iter())
            .map(|(offset, size)| offset + size);

        let data_iter = data_offsets
            .iter()
            .zip(data_sizes.iter())
            .map(|(offset, size)| offset + size);

        let total_size = text_iter.chain(data_iter).max().ok_or_else(|| {
            ParseProblem::InvalidHeader(
                "unable to determine executable size",
                std::panic::Location::current(),
            )
        })?;

        input.goto(base)?;
        let data = input.read_as_vec(total_size as usize)?;
        Ok(Self { data })
    }

    /// Get the executable data.
    pub fn data(&self) -> &[u8] { &self.data }
}
