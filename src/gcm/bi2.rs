//! [GCM][`crate::gcm`] Boot information (`bi2.bin`). Directly follows the boot
//! header (at 0x440) and is always 0x2000 bytes. It seems to contain
//! information about optionss that are passed to the Boot Stage and
//! Apploader[^note-bi2].
//!
//! [^note-bi2]: This implementation assume that there are 0x800 individual options that
//! can be set to any value. This is not necessarily true, the structure of this
//! file is not well understood. Only the first 0x28 bytes are known to be
//! used.

use std::collections::HashMap;

use crate::helper::{Parser, Writer};
use crate::Result;

/// [`Bi2`] Options.
#[derive(Debug, PartialEq, Eq, Copy, Clone, Hash, PartialOrd, Ord)]
pub enum Bi2Options {
    /// Debug monitor size  (unknown purpose).
    DebugMonitorSize,

    /// Simulated memory size (unknown purpose).
    SimulatedMemorySize,

    /// Argument offset (unknown purpose).
    ArgumentOffset,

    /// Debug flag. This indicates whether the game is running in debug mode.
    DebugFlag,

    /// Track location (unknown purpose).
    TrackLocation,

    /// Track size (unknown purpose).
    TrackSize,

    /// Country code (unknown purpose, this already exists in `boot.bin`).
    CountryCode,

    /// Initial PAD_SPEC_X that the Pad library uses.
    PadSpec,

    /// Long filename support (unknown purpose).
    LongFilenameSupport,

    /// Dol limit (unknown purpose).
    DolLimit,

    /// Other unknown options.
    Unknown(usize),
}

impl Bi2Options {
    /// Get the index of a options.
    pub fn index(&self) -> usize {
        use Bi2Options::*;
        match self {
            DebugMonitorSize => 1,
            SimulatedMemorySize => 2,
            ArgumentOffset => 3,
            DebugFlag => 4,
            TrackLocation => 5,
            TrackSize => 6,
            CountryCode => 7,
            PadSpec => 8,
            LongFilenameSupport => 9,
            DolLimit => 11,
            Unknown(index) => *index,
        }
    }
}

impl From<usize> for Bi2Options {
    fn from(index: usize) -> Self {
        use Bi2Options::*;
        match index {
            1 => DebugMonitorSize,
            2 => SimulatedMemorySize,
            3 => ArgumentOffset,
            4 => DebugFlag,
            5 => TrackLocation,
            6 => TrackSize,
            7 => CountryCode,
            8 => PadSpec,
            9 => LongFilenameSupport,
            11 => DolLimit,
            _ => Unknown(index),
        }
    }
}

/// [GCM][`crate::gcm`] Boot information (`bi2.bin`) object.
#[derive(Debug, Default)]
pub struct Bi2 {
    // TODO: HashMap or Array?
    options: HashMap<Bi2Options, u32>,
}

impl Bi2 {
    /// Get options value.
    pub fn get(&self, options: Bi2Options) -> Option<&u32> { self.options.get(&options) }

    /// Set options value.
    pub fn set(&mut self, options: Bi2Options, value: u32) { self.options.insert(options, value); }

    /// Clear options value.
    pub fn clear(&mut self, options: Bi2Options) { self.options.remove(&options); }

    /// Get all options.
    pub fn options(&self) -> &HashMap<Bi2Options, u32> { &self.options }
}

impl Bi2 {
    /// Parse GCM BI2.
    pub fn from_binary<D: Parser>(input: &mut D) -> Result<Self> {
        let options = input
            .bu32_array::<{ 0x2000 / 4 }>()?
            .iter()
            .enumerate()
            .map(|(i, data)| (Bi2Options::from(i), *data))
            .filter(|x| x.1 != 0)
            .collect::<HashMap<_, _>>();

        Ok(Self { options })
    }

    pub fn to_binary<W: Writer>(&self, output: &mut W) -> Result<()> { 
        let mut data = [0u32; 0x2000 / 4];
        for (options, value) in self.options.iter() {
            data[options.index()] = *value;
        }
        output.bu32_array(&data)?;

        Ok(())
    }
}
