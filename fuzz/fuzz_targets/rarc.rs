#![no_main]
use libfuzzer_sys::fuzz_target;
extern crate picori;

fuzz_target!(|data: &[u8]| {
    let reader = std::io::Cursor::new(&data);
    let _ = picori::rarc::RarcReader::new(reader);
});
