#![no_main]
use libfuzzer_sys::fuzz_target;
use picori::string::shift_jis::v1997::IteratorExt as Ie1;
use picori::string::shift_jis::v2004::IteratorExt as Ie2;
extern crate picori;

fuzz_target!(|data: &[u8]| {
    let _ = data.iter().sjis1997().collect::<Result<String, _>>();
    let _ = data.iter().sjis2004().collect::<Result<String, _>>();
});
