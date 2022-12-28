#[cfg(test)]
mod yaz0 {
    use std::io::{Cursor, Read, Seek};

    use picori::yaz0::{self, is_yaz0, Yaz0Reader};

    #[test]
    fn test09() {
        let c = include_bytes!("../assets/tests/yaz0/test.input");
        let d = include_bytes!("../assets/tests/yaz0/test.output");
        let cursor = Cursor::new(c);
        let mut reader = Yaz0Reader::new(cursor).unwrap();
        let mut buf = Vec::with_capacity(reader.decompressed_size());
        let result = reader.read_to_end(&mut buf).unwrap();
        assert_eq!(result, d.len());
        assert_eq!(buf.as_slice(), d);
    }

    #[test]
    fn test2() {
        let c = include_bytes!("../assets/tests/yaz0/test1.input");
        let d = include_bytes!("../assets/tests/yaz0/test1.output");
        let cursor = Cursor::new(c);
        let mut reader = Yaz0Reader::new(cursor).unwrap();
        let mut buf = Vec::with_capacity(reader.decompressed_size());
        let result = reader.read_to_end(&mut buf).unwrap();
        assert_eq!(result, d.len());
        assert_eq!(buf.as_slice(), d);
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

        // If the header is invalid, the Yaz0Reader should be transparent.
        assert!(result.is_ok());
        assert!(result.unwrap().decompressed_size() == 0);
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

        let cursor = Cursor::new(data);
        let result = Yaz0Reader::new(cursor);

        // If the header is invalid, the Yaz0Reader should be transparent.
        assert!(result.is_ok());
        assert!(result.unwrap().decompressed_size() == 0);
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

        let mut cursor = Cursor::new(&data);
        let mut output = [0_u8; 10];
        let result = yaz0::decompress_into(&mut cursor, &mut output);
        assert!(result.is_err());

        let mut cursor = Cursor::new(&data);
        let header = yaz0::Header::from_binary(&mut cursor).unwrap();
        let result = yaz0::decompress(&mut cursor, header.decompressed_size as usize);
        assert!(result.is_err());
    }

    #[test]
    fn seek() {
        let c = include_bytes!("../assets/tests/yaz0/test1.input");
        let d = include_bytes!("../assets/tests/yaz0/test1.output");
        let cursor = Cursor::new(c);
        let mut reader = Yaz0Reader::new(cursor).unwrap();

        let result = reader.seek(std::io::SeekFrom::End(1));
        assert!(result.is_ok());
        reader.seek(std::io::SeekFrom::Start(0)).unwrap();
        let result = reader.seek(std::io::SeekFrom::Current(-1));
        assert!(result.is_err());

        reader.seek(std::io::SeekFrom::Start(0)).unwrap();
        reader
            .seek(std::io::SeekFrom::Start((d.len() / 2) as u64))
            .unwrap();
        reader.seek(std::io::SeekFrom::Current(0)).unwrap();
        reader
            .seek(std::io::SeekFrom::End(-((d.len() / 2) as i64)))
            .unwrap();

        let mut buf = Vec::with_capacity(reader.decompressed_size());
        let result = reader.read_to_end(&mut buf).unwrap();
        assert_eq!(result, d.len() / 2);
        assert_eq!(buf.as_slice(), &d[d.len() / 2..]);
    }
}
