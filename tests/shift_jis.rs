use picori::string::shift_jis::ShiftJisDecoding;

static TEST_UTF8: &[u8] = include_bytes!("../assets/test.txt");
static TEST_SHIFT_JIS: &[u8] = include_bytes!("../assets/test.sjis.txt");

#[cfg(test)]
mod shift_jis {
    use super::*;

    #[test]
    fn test1() {
        let utf8 = String::from_utf8(TEST_UTF8.to_vec()).unwrap();
        let shift_jis = String::from_shift_jis(
            TEST_SHIFT_JIS,
            picori::string::shift_jis::ShiftJisStandard::X2004,
        )
        .unwrap();

        assert_eq!(utf8, shift_jis);
    }
}
