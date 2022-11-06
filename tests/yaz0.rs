#[cfg(test)]
mod yaz0 {
    use std::io::Cursor;

    use picori::yaz0::{is_yaz0, Yaz0Reader};

    #[test]
    fn test09() {
        let c = include_bytes!("../assets/tests/yaz0/test.input");
        let d = include_bytes!("../assets/tests/yaz0/test.output");
        let mut reader = Yaz0Reader::new(Cursor::new(c)).unwrap();
        let result = reader.decompress().unwrap();
        assert_eq!(result, d);
    }

    #[test]
    fn test2() {
        let c = include_bytes!("../assets/tests/yaz0/test1.input");
        let d = include_bytes!("../assets/tests/yaz0/test1.output");
        let mut reader = Yaz0Reader::new(Cursor::new(c)).unwrap();
        let result = reader.decompress().unwrap();
        assert_eq!(result, d);
    }

    #[test]
    fn bad_magic() {
        let data: &[u8] = &[
            0x54, 0x65, 0x73, 0x74, // magic = 'Test'
            0xc0, 0xde, 0xc0, 0xde, // length = 0xde0cde0c
            0x00, 0x00, 0x00, 0x00, // reserved0 = 0x00000000
            0x00, 0x00, 0x00, 0x00, // reserved1 = 0x00000000
        ];

        let cursor = Cursor::new(&data);
        let result = Yaz0Reader::new(cursor);
        assert!(result.is_err());
    }

    #[test]
    fn bad_header() {
        let data: &[u8] = &[
            0x59, 0x61, 0x7A, 0x30, // magic = 'Yaz0'
            0xc0, 0xde, 0xc0, 0xde, // length = 0xde0cde0c
            0x00, 0x00, 0x00,
            0x00, /* reserved0 = 0x00000000
                   * reserved1 = <missing> */
        ];

        let cursor = Cursor::new(&data);
        let result = Yaz0Reader::new(cursor);
        assert!(result.is_err());
    }

    #[test]
    fn decompress_size() {
        let data: &[u8] = &[
            0x59, 0x61, 0x7A, 0x30, // magic = 'Yaz0'
            0xc0, 0xde, 0xc0, 0xde, // length = 0xc0dec0de
            0x00, 0x00, 0x00, 0x00, // reserved0 = 0x00000000
            0x00, 0x00, 0x00, 0x00, // reserved1 = 0x00000000
        ];

        let cursor = Cursor::new(&data);
        let result = Yaz0Reader::new(cursor);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().decompressed_size(), 0xc0dec0de);
    }

    #[test]
    fn check_compressed() {
        let data: &[u8] = &[
            0x59, 0x61, 0x7A, 0x30, // magic = 'Yaz0'
            0xc0, 0xde, 0xc0, 0xde, // length = 0xc0dec0de
            0x00, 0x00, 0x00, 0x00, // reserved0 = 0x00000000
            0x00, 0x00, 0x00, 0x00, // reserved1 = 0x00000000
        ];

        let mut cursor = Cursor::new(data);
        let result = is_yaz0(&mut cursor);
        assert!(result);
    }

    #[test]
    fn small_decompress_size() {
        let data: &[u8] = &[
            0x59, 0x61, 0x7A, 0x30, // magic = 'Yaz0'
            0xc0, 0xde, 0xc0, 0xde, // length = 0xc0dec0de
            0x00, 0x00, 0x00, 0x00, // reserved0 = 0x00000000
            0x00, 0x00, 0x00, 0x00, // reserved1 = 0x00000000
        ];

        let cursor = Cursor::new(&data);
        let mut result = Yaz0Reader::new(cursor).unwrap();
        let mut output = [0_u8; 10];
        let result = result.decompress_into(&mut output);
        assert!(result.is_err());
    }
}
