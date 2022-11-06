#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use picori::format::dol::{from_bytes, section_name, SectionKind};

    #[test]
    fn invalid_header_size() {
        let mut dol = Vec::new();
        dol.extend_from_slice(&[0; 1]);
        assert!(from_bytes(&mut Cursor::new(dol)).is_err());
    }

    #[test]
    fn header() {
        let text_offset: [u32; 7] = [1, 2, 3, 4, 5, 6, 7];
        let text_address: [u32; 7] = [1, 2, 3, 4, 5, 6, 7];
        let text_size: [u32; 7] = [0, 0, 0, 0, 0, 0, 0];
        let data_offset: [u32; 11] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11];
        let data_address: [u32; 11] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11];
        let data_size: [u32; 11] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

        let mut dol = Vec::new();

        for offset in &text_offset {
            dol.extend_from_slice(&offset.to_be_bytes());
        }

        for offset in &data_offset {
            dol.extend_from_slice(&offset.to_be_bytes());
        }

        for address in &text_address {
            dol.extend_from_slice(&address.to_be_bytes());
        }

        for address in &data_address {
            dol.extend_from_slice(&address.to_be_bytes());
        }

        for size in &text_size {
            dol.extend_from_slice(&size.to_be_bytes());
        }

        for size in &data_size {
            dol.extend_from_slice(&size.to_be_bytes());
        }

        dol.extend_from_slice(&0x19876543_u32.to_be_bytes());
        dol.extend_from_slice(&0x29876543_u32.to_be_bytes());
        dol.extend_from_slice(&0x39876543_u32.to_be_bytes());

        dol.extend_from_slice(&[
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ]);

        println!("dol {} bytes", dol.len());

        let dol = from_bytes(&mut Cursor::new(dol)).unwrap();
        assert_eq!(dol.header.text_offset, text_offset);
        assert_eq!(dol.header.text_address, text_address);
        assert_eq!(dol.header.text_size, text_size);
        assert_eq!(dol.header.data_offset, data_offset);
        assert_eq!(dol.header.data_address, data_address);
        assert_eq!(dol.header.data_size, data_size);
        assert_eq!(dol.header.bss_address, 0x19876543);
        assert_eq!(dol.header.bss_size, 0x29876543);
        assert_eq!(dol.header.entry_point, 0x39876543);
        assert_eq!(dol.entry_point(), 0x39876543);
    }

    #[test]
    fn invalid_section_size() {
        let mut dol = Vec::new();
        dol.extend_from_slice(&[1; 0x90]);
        dol.extend_from_slice(&[0; 0x70]);
        let text0_size = 18 * 4 + 18 * 4;
        let text0_size_data = &[0xff, 0xff, 0xff, 0xff];
        dol.splice(text0_size..text0_size + 4, text0_size_data.iter().copied());
        assert!(from_bytes(&mut Cursor::new(dol)).is_err());
    }

    #[test]
    #[should_panic]
    fn section_name_text_unreachable() { section_name(SectionKind::Text, 7); }

    #[test]
    #[should_panic]
    fn section_name_data_unreachable() { section_name(SectionKind::Data, 11); }

    #[test]
    #[should_panic]
    fn section_name_bss_unreachable() { section_name(SectionKind::Bss, 3); }
}
