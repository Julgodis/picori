#![doc(
    html_logo_url = "https://github.com/Julgodis/picori/raw/master/assets/images/picori_logo_512.png"
)]
#![doc(html_favicon_url = "")]
#![doc(html_root_url = "https://docs.rs/picori")]

//! # Picori
//!
//! Picori (ピッコル) is a library for decompilation, modding, and rom-hacking
//! with focus on GameCube and Wii games. It support parsing and building common
//! file formats, e.g., Dolphin executables (DOLs).
//!
//! # Usage
//!
//! Here is a simple example of how to use Picori to parse a DOL file (other
//! formats have examples in their respective modules):
//!
//! ```no_run
//! # use std::fs::File;
//! # use picori::Result;
//! fn main() -> Result<()> {
//!     let mut file = File::open("main.dol")?;
//!     let dol = picori::Dol::from_binary(&mut file)?;
//!     println!("entry point: {:#08x}", dol.entry_point());
//!     Ok(())
//! }
//! ```
//!
//! # Examples
//!
//! The `examples` directory contains a few examples of how to use
//! Picori.
//!
//! * `dol_dump` - Dump information about a `.dol` file.
//! * `rel_dump` - Dump information about a `.rel` file.
//! * `gcm_dump` - Dump information about a `.gcm`/`.iso` file.
//!
//! # Features
//!
//! The following is a list of features supported. More features will be added
//! over time, with the goal of supporting all common file formats used in
//! GameCube and Wii games.
//!
//! * [DOL][crate::dol] - Dolphin executable
//! * [REL][crate::rel] - Relocatable module
//! * [GCM][crate::gcm] - GameCube master disc
//! * [CISO][crate::ciso] - Compact ISO
//! * [Yaz0][crate::yaz0] - Yaz0 compression
//! * [JIX X 0201][crate::jis_x_0201] - JIX X 0201 encoding
//! * [Shift JIS 1997][crate::shift_jis_1997] - Shift JIS 1997 encoding
//! * [Shift JIS 2004][crate::shift_jis_2004] - Shift JIS 2004 encoding

#![deny(missing_docs)]
#![deny(unused_imports)]

pub mod ascii;
pub mod ciso;
pub mod dol;
pub mod gcm;
pub mod jis_x_0201;
pub mod rel;
pub mod shift_jis_1997;
pub mod shift_jis_2004;
pub mod yaz0;

#[doc(inline)]
pub use ascii::{Ascii, IteratorExt as AsciiIteratorExt};
#[doc(inline)]
pub use ciso::CisoReader;
#[doc(inline)]
pub use dol::Dol;
#[doc(inline)]
pub use gcm::Gcm;
#[doc(inline)]
pub use helper::{Error, Result};
#[doc(inline)]
pub use jis_x_0201::{IteratorExt as JisX0201IteratorExt, JisX0201};
#[doc(inline)]
pub use rel::Rel;
#[doc(inline)]
pub use shift_jis_1997::{IteratorExt as ShiftJis1997IteratorExt, ShiftJis1997};
#[doc(inline)]
pub use shift_jis_2004::{IteratorExt as ShiftJis2004IteratorExt, ShiftJis2004};
#[doc(inline)]
pub use yaz0::Yaz0Reader;

mod helper;

/// The [Error][`Error`] enum can indicate a variety of errors that can occur.
/// This module contains the error types that can be returned by this library.
pub mod error {
    pub use super::helper::{
        BuildProblem, CompressionProblem, DecodingProblem, DecompressionProblem, EncodingProblem,
        ParseProblem,
    };
}
