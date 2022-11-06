use anyhow::{ensure, Result};

use super::Decodable;

pub struct Boot {
    pub console_id: u8,             // 0x000 0x001
    pub game_code: [u8; 2],         // 0x002 0x002
    pub country_code: u8,           // 0x003 0x001
    pub maker_code: [u8; 2],        // 0x004 0x002
    pub disc_id: u8,                // 0x006 0x001
    pub version: u8,                // 0x007 0x001
    pub audio_streaming: u8,        // 0x008 0x001
    pub streaming_buffer_size: u8,  // 0x009 0x001
    pub reserved0: [u8; 0x12],      // 0x00A 0x012
    pub magic: u32,                 // 0x01C 0x004
    pub game_name: String,          // 0x020 0x3E0
    pub debug_monitor_offset: u32,  // 0x400 0x004
    pub debug_monitor_address: u32, // 0x404 0x004
    pub reserved1: [u8; 0x18],      // 0x408 0x018
    pub debug_monitor_size: u32,    // 0x420 0x004
    pub fst_offset: u32,            // 0x424 0x004
    pub fst_size: u32,              // 0x428 0x004
    pub fst_max_size: u32,          // 0x42C 0x004
    pub user_position: u32,         // 0x430 0x004
    pub user_length: u32,           // 0x434 0x004
    pub reserved2: [u8; 0x8],       // 0x438 0x008
}

fn utf8_string(input: &[u8]) -> Result<String> { Ok(String::from("test")) }

fn shiftjs_string(input: &[u8]) -> Result<String> { Ok(String::from("test")) }

impl Decodable<Boot> for Boot {
    fn identify_as(input: &[u8]) -> bool {
        return false;
    }

    fn from_bytes(input: &[u8]) -> Result<Self> {
        ensure!(input.len() >= 0x0440, "Boot header is too short");

        let country_code = input[0x003];
        let game_name = match country_code as char {
            'E' => utf8_string(&input[0x020..0x400]),
            'J' => shiftjs_string(&input[0x020..0x400]),
            _ => unimplemented!(),
        };

        unimplemented!()
    }
}
