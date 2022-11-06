#![no_main]
use libfuzzer_sys::fuzz_target;
use picori::ShiftJis1997IteratorExt;
use picori::ShiftJis2004IteratorExt;
extern crate picori;

fuzz_target!(|data: &[u8]| {
    let _ = data.iter().sjis1997().collect::<Result<String, _>>();
    let _ = data.iter().sjis2004().collect::<Result<String, _>>();
});
