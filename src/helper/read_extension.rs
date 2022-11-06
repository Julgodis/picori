
// pub trait ReadExtensionU8: Read {
// fn read_u8(&mut self) -> Result<u8, PicoriError> {
// let mut buf = MaybeUninit::<[u8; 1]>::uninit();
// let slice = unsafe { &mut *buf.as_mut_ptr() };
// self.read_exact(slice)?;
// Ok(unsafe { buf.assume_init()[0] })
// }
// }
// pub trait ReadExtensionU16: Read {
// fn read_eu16<T: EndianAgnostic>(&mut self) -> Result<u16, PicoriError> {
// let mut buf = MaybeUninit::<[u8; 2]>::uninit();
// let slice = unsafe { &mut *buf.as_mut_ptr() };
// self.read_exact(slice)?;
// Ok(T::as_u16(unsafe { &buf.assume_init() }))
// }
//
// #[inline]
// fn read_u16(&mut self) -> Result<u16, PicoriError> {
// self.read_eu16::<NativeEndian>() } #[inline]
// fn read_bu16(&mut self) -> Result<u16, PicoriError> {
// self.read_eu16::<BigEndian>() } #[inline]
// fn read_lu16(&mut self) -> Result<u16, PicoriError> {
// self.read_eu16::<LittleEndian>() } }
//
// pub trait ReadExtensionU32: Read {
// fn read_eu32<T: EndianAgnostic>(&mut self) -> Result<u32, PicoriError> {
// let mut buf = MaybeUninit::<[u8; 4]>::uninit();
// let slice = unsafe { &mut *buf.as_mut_ptr() };
// self.read_exact(slice)?;
// Ok(T::as_u32(unsafe { &buf.assume_init() }))
// }
//
// #[inline]
// fn read_u32(&mut self) -> Result<u32, PicoriError> {
// self.read_eu32::<NativeEndian>() } #[inline]
// fn read_bu32(&mut self) -> Result<u32, PicoriError> {
// self.read_eu32::<BigEndian>() } #[inline]
// fn read_lu32(&mut self) -> Result<u32, PicoriError> {
// self.read_eu32::<LittleEndian>() } }
//
// pub trait ReadArrayExtensionU8: Read {
// fn read_u8_array<const S: usize>(&mut self) -> Result<[u8; S], PicoriError> {
// let mut storage = MaybeUninit::<[u8; S]>::uninit();
// let slice = unsafe { &mut *storage.as_mut_ptr() };
// self.read_exact(slice)?;
// Ok(unsafe { storage.assume_init() })
// }
// }
//
// pub trait ReadArrayExtensionU16: Read {
// fn read_eu16_array<T: EndianAgnostic, const S: usize>(
// &mut self,
// ) -> Result<[u16; S], PicoriError> {
// let mut storage = MaybeUninit::<[u16; S]>::uninit();
// let reference = unsafe { &mut *storage.as_mut_ptr() };
// let buf =
// unsafe { std::slice::from_raw_parts_mut(reference.as_mut_ptr() as *mut u8, 2
// * S) }; self.read_exact(buf)?;
//
// let storage = unsafe { &mut storage.assume_init() };
// for i in 0..S {
// storage[i] = T::as_u16(&buf[2 * i..2 * (i + 1)]);
// }
//
// Ok(*storage)
// }
//
// #[inline]
// fn read_u16_array<const S: usize>(&mut self) -> Result<[u16; S], PicoriError>
// { self.read_eu16_array::<NativeEndian, S>()
// }
// #[inline]
// fn read_bu16_array<const S: usize>(&mut self) -> Result<[u16; S],
// PicoriError> { self.read_eu16_array::<BigEndian, S>()
// }
// #[inline]
// fn read_lu16_array<const S: usize>(&mut self) -> Result<[u16; S],
// PicoriError> { self.read_eu16_array::<LittleEndian, S>()
// }
// }
//
// pub trait ReadArrayExtensionU32: Read {
// fn read_eu32_array<T: EndianAgnostic, const S: usize>(
// &mut self,
// ) -> Result<[u32; S], PicoriError> {
// let mut storage = MaybeUninit::<[u32; S]>::uninit();
// let reference = unsafe { &mut *storage.as_mut_ptr() };
// let buf =
// unsafe { std::slice::from_raw_parts_mut(reference.as_mut_ptr() as *mut u8, 4
// * S) }; self.read_exact(buf)?;
//
// let storage = unsafe { &mut storage.assume_init() };
// for i in 0..S {
// storage[i] = T::as_u32(&buf[4 * i..4 * (i + 1)]);
// }
//
// Ok(*storage)
// }
//
// #[inline]
// fn read_u32_array<const S: usize>(&mut self) -> Result<[u32; S], PicoriError>
// { self.read_eu32_array::<NativeEndian, S>()
// }
// #[inline]
// fn read_bu32_array<const S: usize>(&mut self) -> Result<[u32; S],
// PicoriError> { self.read_eu32_array::<BigEndian, S>()
// }
// #[inline]
// fn read_lu32_array<const S: usize>(&mut self) -> Result<[u32; S],
// PicoriError> { self.read_eu32_array::<LittleEndian, S>()
// }
// }
// pub trait ReadExtension:
// ReadExtensionU8
// + ReadExtensionU16
// + ReadExtensionU32
// + ReadArrayExtensionU8
// + ReadArrayExtensionU16
// + ReadArrayExtensionU32
// {
// }
//
// impl<T: Read + ?Sized> ReadExtensionU8 for T {}
// impl<T: Read + ?Sized> ReadExtensionU16 for T {}
// impl<T: Read + ?Sized> ReadExtensionU32 for T {}
// impl<T: Read + ?Sized> ReadArrayExtensionU8 for T {}
// impl<T: Read + ?Sized> ReadArrayExtensionU16 for T {}
// impl<T: Read + ?Sized> ReadArrayExtensionU32 for T {}
// impl<T: Read + ?Sized> ReadExtension for T {}
