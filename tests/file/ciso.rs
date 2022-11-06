#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use picori::file::ciso::Reader;

    #[test]
    fn reader() {
        // example ciso
        let mut ciso = Vec::<u8>::new();
        ciso.extend_from_slice(&[0x4F, 0x53, 0x49, 0x43]);
        ciso.extend_from_slice(&[0x00, 0x00, 0x00, 0x04]);
        ciso.extend_from_slice(&[0_u8; 0x7FF8]);

        ciso[8 + 1] = 1;
        ciso.extend_from_slice(&[1, 1, 1, 1]);

        ciso[8 + 9] = 1;
        ciso.extend_from_slice(&[9, 9, 9, 9]);

        let mut reader = Cursor::new(ciso);
        let mut decoder = Reader::new(&mut reader).unwrap();
        assert!(decoder.block_size() == 0x04);
        assert!(decoder.total_size() == 0x04 * 10);

        let mut output = Vec::new();
        decoder.decompress(&mut output).unwrap();
        assert_eq!(output.len(), 0x04 * 10);
        assert_eq!(output[0..4], [0, 0, 0, 0]);
        assert_eq!(output[4..8], [1, 1, 1, 1]);
        assert_eq!(output[8..12], [0, 0, 0, 0]);
        assert_eq!(output[12..16], [0, 0, 0, 0]);
        assert_eq!(output[16..20], [0, 0, 0, 0]);
        assert_eq!(output[20..24], [0, 0, 0, 0]);
        assert_eq!(output[24..28], [0, 0, 0, 0]);
        assert_eq!(output[28..32], [0, 0, 0, 0]);
        assert_eq!(output[32..36], [0, 0, 0, 0]);
        assert_eq!(output[36..40], [9, 9, 9, 9]);
    }

    #[test]
    fn invalid_magic() {
        let mut ciso = Vec::<u8>::new();
        ciso.extend_from_slice(&[0, 0, 0, 0]);
        ciso.extend_from_slice(&[0x00, 0x00, 0x00, 0x04]);
        ciso.extend_from_slice(&[1_u8; 0x7FF8]);
        assert!(Reader::new(&mut Cursor::new(ciso)).is_err());
    }

    #[test]
    fn invalid_block_size() {
        let mut ciso = Vec::<u8>::new();
        ciso.extend_from_slice(&[0x4F, 0x53, 0x49, 0x43]);
        ciso.extend_from_slice(&[0, 0, 0, 0]);
        ciso.extend_from_slice(&[1_u8; 0x7FF8]);
        assert!(Reader::new(&mut Cursor::new(ciso)).is_err());

        let mut ciso = Vec::<u8>::new();
        ciso.extend_from_slice(&[0x4F, 0x53, 0x49, 0x43]);
        ciso.extend_from_slice(&[0xFF, 0xFF, 0xFF, 0xFF]);
        ciso.extend_from_slice(&[1_u8; 0x7FF8]);
        assert!(Reader::new(&mut Cursor::new(ciso)).is_err());
    }

    #[test]
    fn invalid_blocks() {
        let mut ciso = Vec::<u8>::new();
        ciso.extend_from_slice(&[0x4F, 0x53, 0x49, 0x43]);
        ciso.extend_from_slice(&[0x00, 0x00, 0x00, 0x04]);
        ciso.extend_from_slice(&[0_u8; 0x7FF8]);
        assert!(Reader::new(&mut Cursor::new(ciso)).is_err());
    }

    #[test]
    fn decode_return_error() {
        let mut ciso = Vec::<u8>::new();
        ciso.extend_from_slice(&[0x4F, 0x53, 0x49, 0x43]);
        ciso.extend_from_slice(&[0x00, 0x00, 0x00, 0x04]);
        ciso.extend_from_slice(&[0_u8; 0x7FF8]);

        ciso[8 + 1] = 1;
        ciso.extend_from_slice(&[1, 1, 1, 1]);

        let mut reader = Cursor::new(ciso);
        let mut decoder = Reader::new(&mut reader).unwrap();

        let result = decoder.blocks().count();
        assert_eq!(result, 2);

        let result = decoder.decompress(&mut Cursor::new([0; 4]));
        assert!(result.is_err());
    }
}
