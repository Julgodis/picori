
#[cfg(test)]
mod dol_tests {
    /*
    use picori::internal::SliceReader;

    use super::*;

    static GZLE01: &[u8] = include_bytes!("../assets/tests/dol/test0.dol");
    #[test]
    fn invalid_header_size() {
        let mut reader = SliceReader::new(&GZLE01[0..10]);
        let result = dol::from_bytes(&mut reader);
        assert!(result.is_err());
    }

    #[test]
    fn section_out_of_bounds() {
        let mut reader = SliceReader::new(&GZLE01[0..0x100]);
        let result = dol::from_bytes(&mut reader);
        assert!(result.is_err());
    }

    #[test]
    fn header() {
        let mut reader = SliceReader::new(&GZLE01);
        let result = dol::from_bytes(&mut reader);

        let text_offset: [u32; 7] = [
            0x00000100, 0x00002620, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000,
        ];
        let text_address: [u32; 7] = [
            0x80003100, 0x800056e0, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000,
        ];
        let text_size: [u32; 7] = [
            0x00002520, 0x00332fa0, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000,
        ];
        let data_offset: [u32; 11] = [
            0x003355c0, 0x00335620, 0x00335680, 0x00335820, 0x00335840, 0x0036e580, 0x0039f960,
            0x003a00a0, 0x00000000, 0x00000000, 0x00000000,
        ];
        let data_address: [u32; 11] = [
            0x80005620, 0x80005680, 0x80338680, 0x80338820, 0x80338840, 0x80371580, 0x803f60e0,
            0x803f7d00, 0x00000000, 0x00000000, 0x00000000,
        ];
        let data_size: [u32; 11] = [
            0x00000060, 0x00000060, 0x000001a0, 0x00000020, 0x00038d40, 0x000313e0, 0x00000740,
            0x00005220, 0x00000000, 0x00000000, 0x00000000,
        ];

        let dol = result.unwrap();
        assert_eq!(dol.header.text_offset, text_offset);
        assert_eq!(dol.header.text_address, text_address);
        assert_eq!(dol.header.text_size, text_size);
        assert_eq!(dol.header.data_offset, data_offset);
        assert_eq!(dol.header.data_address, data_address);
        assert_eq!(dol.header.data_size, data_size);
    }

    #[test]
    fn sections() {
        let mut reader = SliceReader::new(&GZLE01);
        let result = dol::from_bytes(&mut reader);

        let dol = result.unwrap();
        let sections = &dol.sections;
        assert_eq!(sections.len(), 13);
        assert_eq!(sections[0].address, 0x80003100);
        assert_eq!(sections[1].address, 0x800056e0);
        assert_eq!(sections[2].address, 0x80005620);
        assert_eq!(sections[3].address, 0x80005680);
        assert_eq!(sections[4].address, 0x80338680);
        assert_eq!(sections[5].address, 0x80338820);
        assert_eq!(sections[6].address, 0x80338840);
        assert_eq!(sections[7].address, 0x80371580);
        assert_eq!(sections[8].address, 0x803f60e0);
        assert_eq!(sections[9].address, 0x803f7d00);
        assert_eq!(sections[10].address, 0x803a2960);
        assert_eq!(sections[11].address, 0x803f6820);
        assert_eq!(sections[12].address, 0x803fcf20);
    }*/
}
