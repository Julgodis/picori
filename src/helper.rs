use crate::error::PicoriError;

macro_rules! read_bu32 {
    ($buf:ident, $offset:expr) => {
        u32::from_be_bytes(
            $buf[$offset..($offset + 4)]
                .try_into()
                .expect("slice with incorrect length"),
        )
    };
}

macro_rules! read_lu32 {
    ($buf:ident, $offset:expr) => {
        u32::from_le_bytes(
            $buf[$offset..($offset + 4)]
                .try_into()
                .expect("slice with incorrect length"),
        )
    };
}

macro_rules! read_lu64 {
    ($buf:ident, $offset:expr) => {
        u64::from_le_bytes(
            $buf[$offset..($offset + 8)]
                .try_into()
                .expect("slice with incorrect length"),
        )
    };
}

macro_rules! read_bu32_array {
    ($buf:ident, $offset:expr, $len:expr) => {
        $buf[$offset..($offset + 4 * $len)]
            .chunks_exact(4)
            .map(|chunk| u32::from_be_bytes(chunk.try_into().expect("slice with incorrect length")))
            .collect::<Vec<u32>>()
            .try_into()
            .expect("slice with incorrect length")
    };
}

pub(crate) use {read_bu32, read_bu32_array, read_lu32, read_lu64};

pub fn checked_add(a: u32, b: u32) -> Result<u32, PicoriError> {
    a.checked_add(b).ok_or(PicoriError::IntegerOverflow())
}

pub trait TakeLastN<T> {
    fn take_last_n(&self, n: usize) -> &[T];
}

impl<T> TakeLastN<T> for &[T] {
    fn take_last_n(&self, n: usize) -> &[T] {
        if self.len() < n {
            &self[..]
        } else {
            &self[self.len() - n..]
        }
    }
}

pub fn align_next(n: u32, alignment: u32) -> u32 {
    assert!(
        alignment.is_power_of_two(),
        "Alignment must be a power of two"
    );
    (n + alignment - 1) & !(alignment - 1)
}
