//! Deserialize and Serialize of various Nintendo file formats.
//!
//! Formats supported:
//! - [DOL - Dolphin Executable][`dol`]
//! - [REL - Relocatable Executable][`rel`]
//! - [GCM - GameCube Master Disc][`gcm`]
//! - [CISO - Compact ISO (WIB)][`ciso`]

#[cfg(feature = "ciso")]
pub mod ciso;
#[cfg(feature = "dol")]
pub mod dol;
#[cfg(feature = "gcm")]
pub mod gcm;
#[cfg(feature = "rel")]
pub mod rel;
