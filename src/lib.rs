#![doc(
    html_logo_url = "https://raw.githubusercontent.com/Julgodis/picori/master/assets/images/picori_logo_512.png"
)]
#![doc = include_str!("../README.md")]
// TODO: deny missing_docs once all public items have docs
#![allow(missing_docs)]
#![warn(warnings)]

#[cfg(feature = "ascii")]
mod ascii;
#[cfg(feature = "ciso")]
pub mod ciso;
#[cfg(feature = "dol")]
pub mod dol;
#[cfg(feature = "gcm")]
pub mod gcm;
#[cfg(feature = "jis_x_0201")]
mod jis_x_0201;
#[cfg(feature = "rel")]
pub mod rel;
#[cfg(feature = "shift_jis_1997")]
mod shift_jis_1997;
#[cfg(feature = "shift_jis_2004")]
mod shift_jis_2004;
#[cfg(feature = "yaz0")]
pub mod yaz0;

#[cfg(feature = "ascii")]
pub use ascii::{Ascii, IteratorExt as AsciiIteratorExt};
#[cfg(feature = "dol")]
pub use dol::Dol;
#[cfg(feature = "gcm")]
pub use gcm::Gcm;
#[cfg(feature = "gcm")]
pub use gcm::Fst;
pub use helper::{Deserializer, Error, Reader, Result, Seeker};
#[cfg(feature = "jis_x_0201")]
pub use jis_x_0201::{IteratorExt as JisX0201IteratorExt, JisX0201};
pub use rel::Rel;
#[cfg(feature = "shift_jis_1997")]
pub use shift_jis_1997::{IteratorExt as ShiftJis1997IteratorExt, ShiftJis1997};
#[cfg(feature = "shift_jis_2004")]
pub use shift_jis_2004::{IteratorExt as ShiftJis2004IteratorExt, ShiftJis2004};
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
