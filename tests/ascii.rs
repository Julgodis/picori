#[cfg(test)]
mod ascii {
    use picori::string::{StringDecoder, ascii::AsciiDecoder};

    #[test]
    fn ok() {
        for x in 0..=0x7f {
            let result = AsciiDecoder::decode_bytes(&[x]);
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), format!("{}", x as char));
        }
    }

    #[test]
    fn err() {
        for x in 0x80..=0xff {
            let result = AsciiDecoder::decode_bytes(&[x]);
            assert!(result.is_err());
        }
    }

    #[test]
    fn until_zero() {
        let data = b"abc\0def";
        let first = AsciiDecoder::decode_until_zero(data).unwrap();
        let second = AsciiDecoder::decode_until_zero_iterator(
            data.iter().skip_while(|x| **x != 0).skip(1).map(|x| *x),
        )
        .unwrap();

        assert_eq!(first, "abc");
        assert_eq!(second, "def");
    }
}
