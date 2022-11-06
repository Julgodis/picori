#![no_main]
use libfuzzer_sys::fuzz_target;
extern crate picori;

fuzz_target!(|data: &[u8]| {
    let mut reader = std::io::Cursor::new(&data);
    let _ = picori::gcm::Bi2::from_binary(&mut reader);
});
