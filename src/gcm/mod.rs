//! Parse GameCube master disc (GCM) files.
//!
//! GCM is a direct 1-to-1 copy of the a GameCube disc. It contains the
//! executable code, the data files, and the file system table.
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

pub mod apploader;
pub mod bi2;
pub mod boot;
pub mod executable;
pub mod fst;

use std::io::SeekFrom;

#[doc(inline)]
pub use apploader::*;
#[doc(inline)]
pub use bi2::*;
#[doc(inline)]
pub use boot::*;
#[doc(inline)]
pub use executable::*;
#[doc(inline)]
pub use fst::Fst;

use crate::helper::{ensure, ParseProblem, Parser, ProblemLocation, Seeker};
use crate::Result;

/// `.gcm` file object. Because `.gcm` files take up a lot of space, the [`Gcm`]
/// only contains information about the boot, bi2, apploader, executable, and
/// file string table. File specific data is not included. To get the data for a
/// specific file, use [`Gcm::fst`] to find the file entry. Then use
/// [`fst::Entry::File::offset`] and [`fst::Entry::File::size`] to read the file
/// data yourself.
pub struct Gcm {
    boot:       Boot,
    bi2:        Bi2,
    apploader:  Apploader,
    executable: Executable,
    fst:        Fst,
}

impl Gcm {
    /// Parse GCM file from binary stream.
    pub fn from_binary<D: Parser + Seeker>(reader: &mut D) -> Result<Gcm> {
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
        let fst = Fst::from_binary(reader, boot.fst_size as usize)?;

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
