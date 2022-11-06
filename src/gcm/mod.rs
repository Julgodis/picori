//! Parse GameCube Master Disc (GCM) files. GCM is a direct copy of the GameCube
//! disc.
//!
//! # Parse
//!
//! Parse from binary stream by calling [`Gcm::from_binary`]. On error a
//! [Error][`crate::Error`] is return. Otherwise, the parsing succeeded and you
//! get back a [`Gcm`] struct.
//!
//! ## Example
//!
//! This is an example of how to parse a `.gcm`/`.iso` file.
//!
//! ```no_run
//! # use std::fs::File;
//! # use picori::Result;
//! fn main() -> Result<()> {
//!     let mut file = File::open("game.iso")?;
//!     let _ = picori::Gcm::from_binary(&mut file)?;
//!     Ok(())
//! }
//! ```

mod apploader;
mod bi2;
mod boot;
mod executable;
pub mod fst;

use std::io::SeekFrom;

pub use apploader::*;
pub use bi2::*;
pub use boot::*;
pub use executable::*;
pub use fst::Fst;

use crate::helper::{ensure, Deserializer, ParseProblem, ProblemLocation, Seeker};
use crate::Result;

pub struct Gcm {
    boot:       Boot,
    bi2:        Bi2,
    apploader:  Apploader,
    executable: Executable,
    fst:        Fst,
}

impl Gcm {
    pub fn from_binary<D: Deserializer + Seeker>(reader: &mut D) -> Result<Gcm> {
        let position = reader.position()?;

        let boot = Boot::from_binary(reader)?;
        ensure!(
            position + 0x440 == reader.position()?,
            ParseProblem::InvalidData("invalid boot", std::panic::Location::current())
        );

        let bi2 = Bi2::from_binary(reader)?;
        ensure!(
            position + 0x2440 == reader.position()?,
            ParseProblem::InvalidData("invalid bi2", std::panic::Location::current())
        );

        let apploader = Apploader::from_binary(reader)?;
        ensure!(
            position + 0x2460 + (apploader.data.len() as u64) == reader.position()?,
            ParseProblem::InvalidData("invalid apploader", std::panic::Location::current())
        );

        reader.seek(SeekFrom::Start(
            position + boot.main_executable_offset as u64,
        ))?;
        let executable = Executable::from_binary(reader)?;

        reader.seek(SeekFrom::Start(position + boot.fst_offset as u64))?;
        let fst = Fst::deserialize(reader, boot.fst_size as usize)?;

        Ok(Gcm {
            boot,
            bi2,
            apploader,
            executable,
            fst,
        })
    }

    /// Get reference to [`Boot`] struct.
    pub fn boot(&self) -> &Boot { &self.boot }

    /// Get reference to [`Bi2`] struct.
    pub fn bi2(&self) -> &Bi2 { &self.bi2 }

    /// Get reference to [`Apploader`] struct.
    pub fn apploader(&self) -> &Apploader { &self.apploader }

    /// Get reference to [`Executable`] struct.
    pub fn executable(&self) -> &Executable { &self.executable }

    /// Get reference to [`Fst`] struct.
    pub fn fst(&self) -> &Fst { &self.fst }
}
