#![no_main]
use libfuzzer_sys::fuzz_target;
extern crate picori;

fuzz_target!(|data: &[u8]| {
    let mut reader = picori::internal::SliceReader::new(&data);
    let _ = picori::format::dol::from_bytes(&mut reader);
});
