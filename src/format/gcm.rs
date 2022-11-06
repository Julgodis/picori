use std::collections::HashMap;
use std::io::{Read, Seek, SeekFrom, Write};
use std::mem::MaybeUninit;
use std::ops::{ControlFlow, Try};
use std::path::{self, Path, PathBuf};
use std::str::FromStr;

use crate::helper::{align_next, read_bu32, ReadExtension};
use crate::stream::{DeserializeError, DeserializeStream, Deserializeble};
use crate::string::ascii::{AsciiEncoding, AsciiEncodingTrait};

#[derive(Debug)]
pub struct Boot {
    pub console_id: u8,              // 0x000 0x001
    pub game_code: [u8; 2],          // 0x002 0x002
    pub country_code: u8,            // 0x003 0x001
    pub maker_code: [u8; 2],         // 0x004 0x002
    pub disc_id: u8,                 // 0x006 0x001
    pub version: u8,                 // 0x007 0x001
    pub audio_streaming: u8,         // 0x008 0x001
    pub streaming_buffer_size: u8,   // 0x009 0x001
    pub reserved0: [u8; 0x12],       // 0x00A 0x012
    pub magic: u32,                  // 0x01C 0x004
    pub game_name: String,           // 0x020 0x3E0
    pub debug_monitor_offset: u32,   // 0x400 0x004
    pub debug_monitor_address: u32,  // 0x404 0x004
    pub reserved1: [u8; 0x18],       // 0x408 0x018
    pub main_executable_offset: u32, // 0x420 0x004
    pub fst_offset: u32,             // 0x424 0x004
    pub fst_size: u32,               // 0x428 0x004
    pub fst_max_size: u32,           // 0x42C 0x004
    pub user_position: u32,          // 0x430 0x004
    pub user_length: u32,            // 0x434 0x004
    pub unknown: u32,                // 0x438 0x004
    pub reserved2: [u8; 0x4],        // 0x43C 0x004
}

impl Boot {
    pub fn deserialize<D: ReadExtension + Seek>(input: &mut D) -> Result<Self, DeserializeError> {
        let console_id = input.read_bu8()?;
        let game_code = input.read_bu8_array::<2>()?;
        let country_code = input.read_bu8()?;
        let maker_code = input.read_bu8_array::<2>()?;
        let disc_id = input.read_bu8()?;
        let version = input.read_bu8()?;
        let audio_streaming = input.read_bu8()?;
        let streaming_buffer_size = input.read_bu8()?;
        let reserved0 = input.read_bu8_array::<0x12>()?;
        let magic = input.read_bu32()?;
        let game_name = input.read_string::<0x3E0, AsciiEncoding>()?;
        let debug_monitor_offset = input.read_bu32()?;
        let debug_monitor_address = input.read_bu32()?;
        let reserved1 = input.read_bu8_array::<0x18>()?;
        let main_executable_offset = input.read_bu32()?;
        let fst_offset = input.read_bu32()?;
        let fst_size = input.read_bu32()?;
        let fst_max_size = input.read_bu32()?;
        let user_position = input.read_bu32()?;
        let user_length = input.read_bu32()?;
        let unknown = input.read_bu32()?;
        let reserved2 = input.read_bu8_array::<0x4>()?;

        Ok(Self {
            console_id,
            game_code,
            country_code,
            maker_code,
            disc_id,
            version,
            audio_streaming,
            streaming_buffer_size,
            reserved0,
            magic,
            game_name,
            debug_monitor_offset,
            debug_monitor_address,
            reserved1,
            main_executable_offset,
            fst_offset,
            fst_size,
            fst_max_size,
            user_position,
            user_length,
            unknown,
            reserved2,
        })
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone, Hash)]
pub enum Bi2Index {
    DebugMonitorSize,
    SimulatedMemorySize,
    ArgumentOffset,
    DebugFlag,
    TrackLocation,
    TrackSize,
    CountryCode,
    PadSpec,
    LongFilenameSupport,
    DolLimit,
    Unknown(usize),
}

impl Bi2Index {
    pub fn index(&self) -> usize {
        match self {
            Bi2Index::DebugMonitorSize => 1,
            Bi2Index::SimulatedMemorySize => 2,
            Bi2Index::ArgumentOffset => 3,
            Bi2Index::DebugFlag => 4,
            Bi2Index::TrackLocation => 5,
            Bi2Index::TrackSize => 6,
            Bi2Index::CountryCode => 7,
            Bi2Index::PadSpec => 8,
            Bi2Index::LongFilenameSupport => 9,
            Bi2Index::DolLimit => 11,
            Bi2Index::Unknown(index) => *index,
        }
    }
}

impl From<usize> for Bi2Index {
    fn from(index: usize) -> Self {
        match index {
            1 => Bi2Index::DebugMonitorSize,
            2 => Bi2Index::SimulatedMemorySize,
            3 => Bi2Index::ArgumentOffset,
            4 => Bi2Index::DebugFlag,
            5 => Bi2Index::TrackLocation,
            6 => Bi2Index::TrackSize,
            7 => Bi2Index::CountryCode,
            8 => Bi2Index::PadSpec,
            9 => Bi2Index::LongFilenameSupport,
            11 => Bi2Index::DolLimit,
            _ => Bi2Index::Unknown(index),
        }
    }
}

#[derive(Debug)]
pub struct Bi2 {
    pub values: HashMap<Bi2Index, u32>,
}

impl Bi2 {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    pub fn get(&self, index: Bi2Index) -> Option<&u32> { self.values.get(&index) }
    pub fn set(&mut self, index: Bi2Index, value: u32) { self.values.insert(index, value); }
    pub fn clear(&mut self, index: Bi2Index) { self.values.remove(&index); }

    pub fn debug_monitor_size(&self) -> Option<&u32> { self.get(Bi2Index::DebugMonitorSize) }
    pub fn simulated_memory_size(&self) -> Option<&u32> { self.debug_monitor_size() }
    pub fn argument_offset(&self) -> Option<&u32> { self.get(Bi2Index::ArgumentOffset) }
    pub fn debug_flag(&self) -> Option<&u32> { self.get(Bi2Index::DebugFlag) }
    pub fn track_location(&self) -> Option<&u32> { self.get(Bi2Index::TrackLocation) }
    pub fn track_size(&self) -> Option<&u32> { self.get(Bi2Index::TrackSize) }
    pub fn country_code(&self) -> Option<&u32> { self.get(Bi2Index::CountryCode) }
    pub fn pad_spec(&self) -> Option<&u32> { self.get(Bi2Index::PadSpec) }
    pub fn long_filename_support(&self) -> Option<&u32> { self.get(Bi2Index::LongFilenameSupport) }
    pub fn dol_limit(&self) -> Option<&u32> { self.get(Bi2Index::DolLimit) }
}

impl Bi2 {
    pub fn deserialize<D: ReadExtension + Seek>(input: &mut D) -> Result<Self, DeserializeError> {
        let values = input
            .read_bu32_array::<{ 0x2000 / 4 }>()?
            .iter()
            .enumerate()
            .map(|(i, data)| (Bi2Index::from(i), *data))
            .filter(|x| x.1 != 0)
            .collect::<HashMap<_, _>>();

        Ok(Self { values })
    }
}

#[derive(Debug)]
pub struct Apploader {
    pub date:         String,
    pub entrypoint:   u32,
    pub size:         u32,
    pub trailer_size: u32,
    pub unknown:      u32,
    pub data:         Vec<u8>,
}

impl Apploader {
    pub fn deserialize<D: ReadExtension + Seek>(input: &mut D) -> Result<Self, DeserializeError> {
        let date = input.read_string::<0x10, AsciiEncoding>()?;
        let entrypoint = input.read_bu32()?;
        let size = input.read_bu32()?;
        let trailer_size = input.read_bu32()?;
        let unknown = input.read_bu32()?;

        println!("Apploader: date: {}, entrypoint: 0x{:08X}, size: 0x{:08X}, trailer_size: 0x{:08X}, unknown: 0x{:08X}", date, entrypoint, size, trailer_size, unknown);

        let data_size = (size + trailer_size) as usize;
        let mut data = Vec::<u8>::with_capacity(data_size);
        unsafe {
            data.set_len(data_size);
        }
        input.read_stream(&mut data)?;

        Ok(Self {
            date,
            entrypoint,
            size,
            trailer_size,
            unknown,
            data,
        })
    }
}

#[derive(Debug)]
pub struct MainExecutable {
    pub data: Vec<u8>,
}

impl MainExecutable {
    pub fn deserialize<D: ReadExtension + Seek>(input: &mut D) -> Result<Self, DeserializeError> {
        let base = input.stream_position()?;
        let text_offsets = input.read_bu32_array::<{ 4 * 7 }>()?;
        let data_offsets = input.read_bu32_array::<{ 4 * 11 }>()?;
        let text_sizes = input.read_bu32_array::<{ 4 * 7 }>()?;
        let data_sizes = input.read_bu32_array::<{ 4 * 11 }>()?;

        let text_iter = text_offsets
            .iter()
            .zip(text_sizes.iter())
            .map(|(offset, size)| offset + size);

        let data_iter = data_offsets
            .iter()
            .zip(data_sizes.iter())
            .map(|(offset, size)| offset + size);

        let total_size =
            text_iter
                .chain(data_iter)
                .max()
                .ok_or(DeserializeError::InvalidHeader(
                    "unable to find executable size",
                ))?;

        input.seek(SeekFrom::Start(base))?;
        let mut data = Vec::<u8>::with_capacity(total_size as usize);
        unsafe {
            data.set_len(total_size as usize);
        }
        input.read_stream(&mut data)?;

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
    pub fn deserialize<D: ReadExtension + Seek>(input: &mut D) -> Result<Self, DeserializeError> {
        let flag_or_name_offset = input.read_bu32()?;
        let data_offset_or_parent = input.read_bu32()?;
        let data_length_or_end = input.read_bu32()?;
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

pub struct FSTDecoder<'x, Reader: ReadExtension + Seek> {
    pub entries: Vec<FSTEntry>,
    reader:      &'x mut Reader,
    fst_size:    usize,
    base:        u64,
}

impl<'x, Reader: ReadExtension + Seek> FSTDecoder<'x, Reader> {
    pub fn new(reader: &'x mut Reader, fst_size: usize) -> Result<Self, DeserializeError> {
        let base = reader.stream_position()?;
        let mut decoder = Self {
            entries: Vec::new(),
            reader,
            fst_size,
            base,
        };
        decoder.parse_entries()?;
        Ok(decoder)
    }

    fn parse_entries(&mut self) -> Result<(), DeserializeError> {
        let _1 = self.reader.read_bu32()?;
        let _2 = self.reader.read_bu32()?;
        let root_count = self.reader.read_bu32()?;
        let entry_count = root_count as usize;
        if entry_count > 0x2000 {
            return Err(DeserializeError::InvalidHeader("entry count limit reached"));
        }

        let entries = (1..entry_count)
            .map(|_| _FstEntry::deserialize(self.reader))
            .collect::<Result<Vec<_>, _>>()?;

        let entry_size = 0x0C * entry_count;
        let name_table_size = self.fst_size - entry_size;

        let mut string_table = Vec::<u8>::with_capacity(name_table_size);
        unsafe { string_table.set_len(name_table_size) }
        self.reader.read_exact(&mut string_table)?;

        for (i, entry) in entries.iter().enumerate() {
            self.entries.push(match entry {
                _FstEntry::File { name, offset, size } => FSTEntry::File {
                    name:   String::from_ascii(&string_table[*name as usize..])?,
                    index:  i as u32,
                    offset: *offset,
                    size:   *size,
                },
                _FstEntry::Directory { name, parent, end } => FSTEntry::Directory {
                    name:   String::from_ascii(&string_table[*name as usize..])?,
                    parent: *parent,
                    begin:  (i + 1) as u32,
                    end:    *end - 1,
                },
            });
        }

        Ok(())
    }

    pub fn write_file<W: Write>(
        &mut self,
        entry: &FSTEntry,
        writer: &mut W,
    ) -> Result<(), DeserializeError> {
        match entry {
            FSTEntry::File { offset, size, .. } => {
                self.reader.seek(SeekFrom::Start(*offset as u64))?;

                let mut buffer = MaybeUninit::<[u8; 0x1000]>::uninit();
                let mut remaining = *size as isize;
                while remaining > 0 {
                    let read = self.reader.read(unsafe { &mut *buffer.as_mut_ptr() })?;
                    writer.write(unsafe { &buffer.assume_init() })?;
                    remaining -= read as isize;
                }

                Ok(())
            },
            _ => Err(DeserializeError::InvalidData("not a file")),
        }
    }

    pub fn file_data(&mut self, entry: &FSTEntry) -> Result<Vec<u8>, DeserializeError> {
        match entry {
            FSTEntry::File { offset, size, .. } => {
                self.reader.seek(SeekFrom::Start(*offset as u64))?;
                let mut data = Vec::<u8>::with_capacity(*size as usize);
                unsafe { data.set_len(*size as usize) }
                self.reader.read_exact(&mut data)?;
                Ok(data)
            },
            _ => Err(DeserializeError::InvalidData("not a file")),
        }
    }

    pub fn files(&self, directory: FSTEntry) -> Result<Vec<FSTEntry>, DeserializeError> {
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
            Err(DeserializeError::InvalidData("not a directory"))
        }
    }

    pub fn root_directory(&self) -> FSTEntry {
        FSTEntry::Directory {
            name:   String::from(""),
            parent: 0,
            begin:  0,
            end:    self.entries.len() as u32,
        }
    }

    fn for_each_file<F, R>(
        &mut self,
        path: &mut PathBuf,
        index: &mut usize,
        entry: &FSTEntry,
        func: &mut F,
    ) -> R
    where
        F: FnMut(&mut Self, &Path, &FSTEntry) -> R,
        R: Try<Output = ()>,
    {
        match entry {
            FSTEntry::File { name, .. } => path.push(name),
            FSTEntry::Directory { name, .. } => path.push(name),
        };

        func(self, path.as_path(), entry)?;

        if let FSTEntry::Directory { begin, end, .. } = entry {
            *index  = *begin as usize;
            while *index < *end as usize {
                let next_entry = self.entries[*index].clone();
                self.for_each_file(path, index, &next_entry, func)?;
            }
            path.pop();
        } else {
            path.pop();
            *index += 1;
        }

        R::from_output(())
    }

    #[inline]
    pub fn try_for_each<F, R>(&mut self, entry: FSTEntry, func: &mut F) -> R
    where
        F: FnMut(&mut Self, &Path, &FSTEntry) -> R,
        R: Try<Output = ()>,
    {
        let mut path = PathBuf::new();
        let mut index = match &entry {
            FSTEntry::File { .. } => 0,
            FSTEntry::Directory { begin, .. } => *begin as usize,
        };

        self.for_each_file(&mut path, &mut index, &entry, func)
    }

    pub fn for_each<F>(&mut self, entry: FSTEntry, mut func: F)
    where
        F: FnMut(&mut Self, &Path, &FSTEntry),
    {
        self.try_for_each(entry, &mut |decoder, path, entry| {
            func(decoder, path, entry);
            Ok::<(), ()>(())
        })
        .unwrap();
    }

    #[inline]
    pub fn try_for_each_all<F, R>(&mut self, func: &mut F) -> R
    where
        F: FnMut(&mut Self, &Path, &FSTEntry) -> R,
        R: Try<Output = ()>,
    {
        let root = self.root_directory();
        self.try_for_each(root, func)
    }

    #[inline]
    pub fn for_each_all<F>(&mut self, func: F)
    where
        F: FnMut(&mut Self, &Path, &FSTEntry),
    {
        let root = self.root_directory();
        self.for_each(root, func)
    }
}
