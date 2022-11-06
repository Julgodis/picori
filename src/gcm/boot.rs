//! [GCM][`crate::gcm`] Boot Header (`boot.bin`). This is the first 0x440 bytes
//! of the GCM image.

use crate::error::ParseProblem;
use crate::helper::{ensure, Parser, ProblemLocation};
use crate::{Ascii, Result};

/// [`Boot`] Console Type.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ConsoleType {
    /// Nintendo GameCube.
    GameCube,
}

/// [GCM][`crate::gcm`] Boot Header (`boot.bin`) object.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Boot {
    /// Console.
    pub console: ConsoleType,

    /// Game code.
    pub game_code: [u8; 2],

    /// Region code.
    pub country_code: u8,

    /// Maker code.
    pub maker_code: [u8; 2],

    /// Disc number.
    pub disc_id: u8,

    /// Version.
    pub version: u8,

    /// Audio streaming.
    pub audio_streaming: u8,

    /// Stream buffer size.
    pub streaming_buffer_size: u8,

    /// Game name.
    pub game_name: String,

    /// Debug monitor offset (unknown purpose).
    pub debug_monitor_offset: u32,

    /// Debug monitor address (unknown purpose).
    pub debug_monitor_address: u32,

    /// Main executable offset from the start of the file.
    pub main_executable_offset: u32,

    /// FST offset from the start of the file.
    pub fst_offset: u32,

    /// FST size for this disc.
    pub fst_size: u32,

    /// FST max size. FST is shared between all discs and this is
    /// the total size of the FST when all discs are combined.
    pub fst_max_size: u32,

    /// User position (unknown purpose).
    pub user_position: u32,

    /// User length (unknown purpose).
    pub user_length: u32,

    /// Unknown0 (unknown purpose).
    pub unknown0: u32,
}

impl Boot {
    /// Parse GCM Boot.
    pub fn from_binary<D: Parser>(input: &mut D) -> Result<Self> {
        let console = input.deserialize_u8()?;
        let game_code = input.deserialize_u8_array::<2>()?;
        let country_code = input.deserialize_u8()?;
        let maker_code = input.deserialize_u8_array::<2>()?;
        let disc_id = input.deserialize_u8()?;
        let version = input.deserialize_u8()?;
        let audio_streaming = input.deserialize_u8()?;
        let streaming_buffer_size = input.deserialize_u8()?;
        let _reserved0 = input.deserialize_u8_array::<0x12>()?;
        let magic = input.deserialize_bu32()?;
        let game_name = input.deserialize_str::<0x3E0, Ascii>()?;
        let debug_monitor_offset = input.deserialize_bu32()?;
        let debug_monitor_address = input.deserialize_bu32()?;
        let _reserved1 = input.deserialize_u8_array::<0x18>()?;
        let main_executable_offset = input.deserialize_bu32()?;
        let fst_offset = input.deserialize_bu32()?;
        let fst_size = input.deserialize_bu32()?;
        let fst_max_size = input.deserialize_bu32()?;
        let user_position = input.deserialize_bu32()?;
        let user_length = input.deserialize_bu32()?;
        let unknown0 = input.deserialize_bu32()?;
        let _reserved2 = input.deserialize_u8_array::<0x4>()?;

        ensure!(
            magic == 0xC2339F3D,
            ParseProblem::InvalidHeader("invalid magic", std::panic::Location::current())
        );

        let console = match console {
            0x47 => ConsoleType::GameCube,
            _ => Err(ParseProblem::InvalidHeader(
                "invalid console type",
                std::panic::Location::current(),
            ))?,
        };

        Ok(Self {
            console,
            game_code,
            country_code,
            maker_code,
            disc_id,
            version,
            audio_streaming,
            streaming_buffer_size,
            game_name,
            debug_monitor_offset,
            debug_monitor_address,
            main_executable_offset,
            fst_offset,
            fst_size,
            fst_max_size,
            user_position,
            user_length,
            unknown0,
        })
    }
}
