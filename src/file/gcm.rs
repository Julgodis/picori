use std::collections::HashMap;
use std::io::{SeekFrom, Write};

use crate::encoding::Ascii;
use crate::error::DeserializeProblem::*;
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FSTEntry {
    File {
        name:   String,
        index:  u32,
        offset: u32,
        size:   u32,
    },
    Directory {
        name:   String,
        parent: u32,
        begin:  u32,
        end:    u32,
    },
}

enum _FstEntry {
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

impl _FstEntry {
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

pub struct FSTDecoder<'x, D: Deserializer + Seeker> {
    pub entries: Vec<FSTEntry>,
    reader:      &'x mut D,
    fst_size:    usize,
}

impl<'x, D: Deserializer + Seeker> FSTDecoder<'x, D> {
    #[inline]
    pub fn new(reader: &'x mut D, fst_size: usize) -> Result<Self> {
        let mut decoder = Self {
            entries: Vec::new(),
            reader,
            fst_size,
        };
        decoder.parse_entries()?;
        Ok(decoder)
    }

    fn parse_entries(&mut self) -> Result<()> {
        let _ = self.reader.deserialize_bu32()?;
        let _ = self.reader.deserialize_bu32()?;
        let root_count = self.reader.deserialize_bu32()?;
        let entry_count = root_count as usize;
        ensure!(
            entry_count <= 0x4000,
            InvalidHeader("entry count limit (max 16384)")
        );

        let entries = (1..entry_count)
            .map(|_| _FstEntry::deserialize(self.reader))
            .collect::<Result<Vec<_>>>()?;

        let entry_size = 0x0C * entry_count;
        let name_table_size = self.fst_size - entry_size;
        let string_table = self.reader.read_buffer(name_table_size)?;

        for (i, entry) in entries.iter().enumerate() {
            self.entries.push(match entry {
                _FstEntry::File { name, offset, size } => FSTEntry::File {
                    name:   Ascii::first(&string_table[*name as usize..])?,
                    index:  i as u32,
                    offset: *offset,
                    size:   *size,
                },
                _FstEntry::Directory { name, parent, end } => FSTEntry::Directory {
                    name:   Ascii::first(&string_table[*name as usize..])?,
                    parent: *parent,
                    begin:  (i + 1) as u32,
                    end:    *end - 1,
                },
            });
        }

        Ok(())
    }

    pub fn write_file<W: Write>(&mut self, entry: &FSTEntry, _writer: &mut W) -> Result<()> {
        match entry {
            FSTEntry::File { offset, .. } => {
                self.reader.seek(SeekFrom::Start(*offset as u64))?;

                todo!()
                /*
                let mut buffer = MaybeUninit::<[u8; 0x1000]>::uninit();
                let mut remaining = *size as isize;
                while remaining > 0 {
                    let read = self.reader.read_into_buffer(unsafe { &mut *buffer.as_mut_ptr() })?;
                    writer.write(unsafe { &buffer.assume_init() })?;
                    remaining -= read as isize;
                }

                Ok(())*/
            },
            _ => Err(InvalidData("not a file").into()),
        }
    }

    #[inline]
    pub fn file_data(&mut self, entry: &FSTEntry) -> Result<Vec<u8>> {
        match entry {
            FSTEntry::File { offset, size, .. } => {
                self.reader.seek(SeekFrom::Start(*offset as u64))?;
                Ok(self.reader.read_buffer(*size as usize)?)
            },
            _ => Err(InvalidData("not a file").into()),
        }
    }

    pub fn files(&self, directory: FSTEntry) -> Result<Vec<FSTEntry>> {
        if let FSTEntry::Directory { begin, end, .. } = directory {
            let mut files = Vec::new();
            let mut index = begin as usize;
            while index < end as usize {
                let entry = &self.entries[index];
                if let FSTEntry::File { .. } = entry {
                    index += 1;
                    files.push(entry.clone());
                } else if let FSTEntry::Directory { end, .. } = entry {
                    index = *end as usize;
                    files.push(entry.clone());
                }
            }

            Ok(files)
        } else {
            Err(InvalidData("not a directory").into())
        }
    }

    #[inline]
    pub fn root_directory(&self) -> FSTEntry {
        FSTEntry::Directory {
            name:   String::from(""),
            parent: 0,
            begin:  0,
            end:    self.entries.len() as u32,
        }
    }

    // fn for_each_file<F, R>(
    // &mut self,
    // path: &mut PathBuf,
    // index: &mut usize,
    // entry: &FSTEntry,
    // func: &mut F,
    // ) -> R
    // where
    // F: FnMut(&mut Self, &Path, &FSTEntry) -> R,
    // R: Try<Output = ()>,
    // {
    // match entry {
    // FSTEntry::File { name, .. } => path.push(name),
    // FSTEntry::Directory { name, .. } => path.push(name),
    // };
    //
    // func(self, path.as_path(), entry)?;
    //
    // if let FSTEntry::Directory { begin, end, .. } = entry {
    // index = *begin as usize;
    // while *index < *end as usize {
    // let next_entry = self.entries[*index].clone();
    // self.for_each_file(path, index, &next_entry, func)?;
    // }
    // path.pop();
    // } else {
    // path.pop();
    // index += 1;
    // }
    //
    // R::from_output(())
    // }
    //
    // #[inline]
    // pub fn try_for_each<F, R>(&mut self, entry: FSTEntry, func: &mut F) -> R
    // where
    // F: FnMut(&mut Self, &Path, &FSTEntry) -> R,
    // R: Try<Output = ()>,
    // {
    // let mut path = PathBuf::new();
    // let mut index = match &entry {
    // FSTEntry::File { .. } => 0,
    // FSTEntry::Directory { begin, .. } => *begin as usize,
    // };
    //
    // self.for_each_file(&mut path, &mut index, &entry, func)
    // }
    //
    // pub fn for_each<F>(&mut self, entry: FSTEntry, mut func: F)
    // where
    // F: FnMut(&mut Self, &Path, &FSTEntry),
    // {
    // self.try_for_each(entry, &mut |decoder, path, entry| {
    // func(decoder, path, entry);
    // Ok::<(), ()>(())
    // })
    // .unwrap();
    // }
    //
    // #[inline]
    // pub fn try_for_each_all<F, R>(&mut self, func: &mut F) -> R
    // where
    // F: FnMut(&mut Self, &Path, &FSTEntry) -> R,
    // R: Try<Output = ()>,
    // {
    // let root = self.root_directory();
    // self.try_for_each(root, func)
    // }
    //
    // #[inline]
    // pub fn for_each_all<F>(&mut self, func: F)
    // where
    // F: FnMut(&mut Self, &Path, &FSTEntry),
    // {
    // let root = self.root_directory();
    // self.for_each(root, func)
    // }
}

pub struct GCM {
    boot: Boot,
    bi2:  Bi2,
    apploader: Apploader,
}

impl GCM {
    pub fn boot(&self) -> &Boot { &self.boot }

    pub fn bi2(&self) -> &Bi2 { &self.bi2 }

    pub fn apploader(&self) -> &Apploader { &self.apploader }
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

    reader.seek(SeekFrom::Start(boot.main_executable_offset as u64))?;

    reader.seek(SeekFrom::Start(boot.fst_offset as u64))?;

    Ok(GCM { boot, bi2, apploader })
}
