//! Deserialize and Serialize GameCube Master Disc (GCM) file. This file format is a direct copy of the
//! GameCube disc.
//!
//! # Deserialize
//!
//! ```no_run
//! # use std::fs::File;
//! # use picori::Result;
//! fn main() -> Result<()> {
//!     let mut file = File::open("game.iso")?;
//!     let _ = picori::file::gcm::parse(&mut file)?;
//!     Ok(())
//! }
//! ```
//!
//! # Serialize
//!
//! TODO: Write this section.

use std::collections::HashMap;
use std::io::SeekFrom;
use std::path::PathBuf;

use super::dol::Dol;
use crate::encoding::Ascii;
use crate::error::DeserializeProblem::*;
use crate::file::dol;
use crate::helper::{ensure, Deserializer, Seeker};
use crate::Result;

/// GCM Boot Header (`boot.bin`). This is the first 0x440 bytes of the GCM
/// image.
#[derive(Debug, Default)]
pub struct Boot {
    /// Console id.
    pub console_id: u8,

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

    /// Disc magic (0xC2339F3D).
    pub magic: u32, // 0x01C 0x004

    /// Game name.
    pub game_name: String, // 0x020 0x3E0

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
    /// Deserialize a GCM Boot from a
    /// [Deserializer][`crate::helper::Deserializer`].
    pub fn deserialize<D: Deserializer>(input: &mut D) -> Result<Self> {
        let console_id = input.deserialize_u8()?;
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

        Ok(Self {
            console_id,
            game_code,
            country_code,
            maker_code,
            disc_id,
            version,
            audio_streaming,
            streaming_buffer_size,
            magic,
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

/// GCM Boot information (`bi2.bin`). Directly follows the boot header and is
/// always 0x2000 bytes. It seems to contain information about optionss that are
/// passed to the Boot Stage and Apploader[^note-bi2].
///
/// [^note-bi2]: This implementation assume that there is 0x800 individual optionss that
/// can be set to any value. This is not necessarily true, the structure of this
/// file is  not well understood. Only the first 0x28 bytes are known to be
/// used.
#[derive(Debug, Default)]
pub struct Bi2 {
    options: HashMap<Bi2Options, u32>,
}

impl Bi2 {
    /// Get options value.
    pub fn get(&self, options: Bi2Options) -> Option<&u32> { self.options.get(&options) }

    /// Set options value.
    pub fn set(&mut self, options: Bi2Options, value: u32) { self.options.insert(options, value); }

    /// Clear options value.
    pub fn clear(&mut self, options: Bi2Options) { self.options.remove(&options); }

    pub fn options(&self) -> &HashMap<Bi2Options, u32> { &self.options }
}

impl Bi2 {
    /// Deserialize a GCM Bi2 from a
    /// [Deserializer][`crate::helper::Deserializer`].
    pub fn deserialize<D: Deserializer>(input: &mut D) -> Result<Self> {
        let options = input
            .deserialize_bu32_array::<{ 0x2000 / 4 }>()?
            .iter()
            .enumerate()
            .map(|(i, data)| (Bi2Options::from(i), *data))
            .filter(|x| x.1 != 0)
            .collect::<HashMap<_, _>>();

        Ok(Self { options })
    }
}

/// GCM Apploader (`apploader.img`). The boot stages loads the Apploader.
///  It is a small program that loads the main executable and the FST.
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
    /// Deserialize a GCM Apploader from a
    /// [Deserializer][`crate::helper::Deserializer`].
    pub fn deserialize<D: Deserializer>(input: &mut D) -> Result<Self> {
        let date = input.deserialize_str::<0x10, Ascii>()?;
        let entry_point = input.deserialize_bu32()?;
        let size = input.deserialize_bu32()?;
        let trailer_size = input.deserialize_bu32()?;
        let unknown = input.deserialize_bu32()?;
        let data_size = (size + trailer_size) as usize;
        let data = input.read_buffer(data_size)?;

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

#[derive(Debug, Default)]
pub struct MainExecutable {
    pub data: Vec<u8>,
}

impl MainExecutable {
    pub fn deserialize<D: Deserializer + Seeker>(input: &mut D) -> Result<Self> {
        let base = input.position()?;
        let text_offsets = input.deserialize_bu32_array::<7>()?;
        let data_offsets = input.deserialize_bu32_array::<11>()?;
        let _ = input.deserialize_bu32_array::<7>()?;
        let _ = input.deserialize_bu32_array::<11>()?;
        let text_sizes = input.deserialize_bu32_array::<7>()?;
        let data_sizes = input.deserialize_bu32_array::<11>()?;

        let text_iter = text_offsets
            .iter()
            .zip(text_sizes.iter())
            .map(|(offset, size)| offset + size);

        let data_iter = data_offsets
            .iter()
            .zip(data_sizes.iter())
            .map(|(offset, size)| offset + size);

        let total_size = text_iter
            .chain(data_iter)
            .max()
            .ok_or(InvalidHeader("unable to find executable size"))?;

        input.seek(SeekFrom::Start(base))?;
        let data = input.read_buffer(total_size as usize)?;
        Ok(Self { data })
    }
}

/// Enum varient of a single [`Fst`] entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FstEntry {
    /// Root directory.
    Root,
    /// File.
    File {
        /// Relative filename.
        name:   String,
        /// `internal`: Entry index.
        index:  u32,
        /// File offset from the beginning of the GCM file.
        offset: u32,
        /// File size.
        size:   u32,
    },
    /// Directory.
    Directory {
        /// Relative directory name.
        name:   String,
        /// `internal`: Parent index.
        parent: u32,
        /// `internal`: First child index.
        begin:  u32,
        /// `internal`: Last child index.
        end:    u32,
    },
}

enum TempFstEntry {
    File {
        name:   u32,
        offset: u32,
        size:   u32,
    },
    Directory {
        name:   u32,
        parent: u32,
        end:    u32,
    },
}

impl TempFstEntry {
    pub fn deserialize<D: Deserializer + Seeker>(input: &mut D) -> Result<Self> {
        let flag_or_name_offset = input.deserialize_bu32()?;
        let data_offset_or_parent = input.deserialize_bu32()?;
        let data_length_or_end = input.deserialize_bu32()?;
        let flag = flag_or_name_offset >> 24;
        let name_offset = flag_or_name_offset & 0x00ffffff;

        if (flag & 1) == 0 {
            Ok(Self::File {
                name:   name_offset,
                offset: data_offset_or_parent,
                size:   data_length_or_end,
            })
        } else {
            Ok(Self::Directory {
                name:   name_offset,
                parent: data_offset_or_parent,
                end:    data_length_or_end,
            })
        }
    }
}

/// File String Table (`fst.bin`). The [`Fst`] contains information about the
/// file structure of the GameCube disc, i.e. the file names and their
/// locations.
pub struct Fst {
    entries: Vec<FstEntry>,
}

impl Fst {
    /// Deserialize a [`Fst`] from a
    /// [Deserializer][`crate::helper::Deserializer`] +
    /// [Seeker][`crate::helper::Seeker`].
    ///
    /// To read the full string table, this function needs the size of the
    /// [`Fst`]. This is available in the [`Boot`] struct.
    pub fn deserialize<D: Deserializer + Seeker>(reader: &mut D, fst_size: usize) -> Result<Fst> {
        let base = reader.position()?;

        let _ = reader.deserialize_bu32()?;
        let _ = reader.deserialize_bu32()?;
        let root_count = reader.deserialize_bu32()?;
        let entry_count = root_count as usize;
        ensure!(
            entry_count <= 0x4000,
            InvalidHeader("entry count limit (max 16384)")
        );

        reader.seek(SeekFrom::Start(base))?;
        let temp_entries = (0..entry_count)
            .map(|_| TempFstEntry::deserialize(reader))
            .collect::<Result<Vec<_>>>()?;

        let entry_size = 0x0C * entry_count;
        let name_table_size = fst_size - entry_size;
        let string_table = reader.read_buffer(name_table_size)?;

        let mut entries = Vec::with_capacity(entry_count);
        for (i, entry) in temp_entries.iter().enumerate() {
            if i == 0 {
                entries.push(FstEntry::Root);
                continue;
            }

            let entry = match entry {
                TempFstEntry::File { name, offset, size } => FstEntry::File {
                    name:   Ascii::first(&string_table[*name as usize..])?,
                    index:  i as u32,
                    offset: *offset,
                    size:   *size,
                },
                TempFstEntry::Directory { name, parent, end } => FstEntry::Directory {
                    name:   Ascii::first(&string_table[*name as usize..])?,
                    parent: *parent,
                    begin:  (i + 1) as u32,
                    end:    *end,
                },
            };

            entries.push(entry);
        }

        Ok(Fst { entries })
    }

    /// Get an iterator over all [`FstEntry`]s.
    pub fn files(&self) -> FstFileIterator {
        FstFileIterator {
            fst: self,
            index: 0,
            last_index_of_directory: vec![],
            path: PathBuf::new(),
        }
    }
}

/// Iterator over all files in a [`Fst`].
pub struct FstFileIterator<'fst> {
    fst: &'fst Fst,
    index: usize,
    last_index_of_directory: Vec<usize>,
    path: PathBuf,
}

impl<'fst> Iterator for FstFileIterator<'fst> {
    type Item = (PathBuf, FstEntry);

    fn next(&mut self) -> Option<Self::Item> {
        while !self.last_index_of_directory.is_empty() {
            if self.last_index_of_directory.last() == Some(&self.index) {
                self.last_index_of_directory.pop();
                self.path.pop();
            } else {
                break;
            }
        }

        let entry = self.fst.entries.get(self.index)?;
        self.index += 1;

        match entry {
            FstEntry::File { name, .. } => {
                let path = self.path.join(name);
                Some((path, entry.clone()))
            },
            FstEntry::Directory {
                begin, end, name, ..
            } => {
                self.index = *begin as usize;
                self.last_index_of_directory.push(*end as usize);
                self.path.push(name);
                Some((self.path.clone(), entry.clone()))
            },
            FstEntry::Root => Some((self.path.clone(), entry.clone())),
        }
    }
}

pub struct GCM {
    boot:      Boot,
    bi2:       Bi2,
    apploader: Apploader,
    dol:       Dol,
    fst:       Fst,
}

impl GCM {
    /// Get reference to [`Boot`] struct.
    pub fn boot(&self) -> &Boot { &self.boot }

    /// Get reference to [`Bi2`] struct.
    pub fn bi2(&self) -> &Bi2 { &self.bi2 }

    /// Get reference to [`Apploader`] struct.
    pub fn apploader(&self) -> &Apploader { &self.apploader }

    /// Get reference to [`Dol`] struct.
    pub fn dol(&self) -> &Dol { &self.dol }

    /// Get reference to [`Fst`] struct.
    pub fn fst(&self) -> &Fst { &self.fst }
}

pub fn parse<D: Deserializer + Seeker>(reader: &mut D) -> Result<GCM> {
    let position = reader.position()?;

    let boot = Boot::deserialize(reader)?;
    ensure!(
        position + 0x440 == reader.position()?,
        InvalidData("invalid boot")
    );

    let bi2 = Bi2::deserialize(reader)?;
    ensure!(
        position + 0x2440 == reader.position()?,
        InvalidData("invalid bi2")
    );

    let apploader = Apploader::deserialize(reader)?;
    ensure!(
        position + 0x2460 + (apploader.data.len() as u64) == reader.position()?,
        InvalidData("invalid apploader")
    );

    reader.seek(SeekFrom::Start(
        position + boot.main_executable_offset as u64,
    ))?;
    let dol = dol::parse(reader)?;

    reader.seek(SeekFrom::Start(position + boot.fst_offset as u64))?;
    let fst = Fst::deserialize(reader, boot.fst_size as usize)?;

    Ok(GCM {
        boot,
        bi2,
        apploader,
        dol,
        fst,
    })
}
