//! [GCM][`crate::gcm`] Boot Header (`boot.bin`). This is the first 0x440 bytes
//! of the GCM image.

use crate::error::ParseProblem;
use crate::helper::{ensure, Parser, ProblemLocation, Writer};
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
        let console = input.u8()?;
        let game_code = input.u8_array::<2>()?;
        let country_code = input.u8()?;
        let maker_code = input.u8_array::<2>()?;
        let disc_id = input.u8()?;
        let version = input.u8()?;
        let audio_streaming = input.u8()?;
        let streaming_buffer_size = input.u8()?;
        let _reserved0 = input.u8_array::<0x12>()?;
        let magic = input.bu32()?;
        let game_name = input.str_fixed::<0x3E0, Ascii>()?;
        let debug_monitor_offset = input.bu32()?;
        let debug_monitor_address = input.bu32()?;
        let _reserved1 = input.u8_array::<0x18>()?;
        let main_executable_offset = input.bu32()?;
        let fst_offset = input.bu32()?;
        let fst_size = input.bu32()?;
        let fst_max_size = input.bu32()?;
        let user_position = input.bu32()?;
        let user_length = input.bu32()?;
        let unknown0 = input.bu32()?;
        let _reserved2 = input.u8_array::<0x4>()?;

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

    pub fn to_binary<W: Writer>(&self, output: &mut W) -> Result<()> {
        let console = match self.console {
            ConsoleType::GameCube => 0x47_u8,
        };

        output.u8(console)?;
        output.u8_array(&self.game_code)?;
        output.u8(self.country_code)?;
        output.u8_array(&self.maker_code)?;
        output.u8(self.disc_id)?;
        output.u8(self.version)?;
        output.u8(self.audio_streaming)?;
        output.u8(self.streaming_buffer_size)?;
        output.u8_array(&[0; 0x12])?;
        output.bu32(0xC2339F3D)?;
        output.str::<0x3E0, Ascii>(&self.game_name)?;
        output.bu32(self.debug_monitor_offset)?;
        output.bu32(self.debug_monitor_address)?;
        output.u8_array(&[0; 0x18])?;
        output.bu32(self.main_executable_offset)?;
        output.bu32(self.fst_offset)?;
        output.bu32(self.fst_size)?;
        output.bu32(self.fst_max_size)?;
        output.bu32(self.user_position)?;
        output.bu32(self.user_length)?;
        output.bu32(self.unknown0)?;
        output.u8_array(&[0; 0x4])?;

        Ok(())
    }
}
