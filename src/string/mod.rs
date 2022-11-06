//! TODO:

mod ascii;
mod jis_x_0201;
mod shift_jis_1997;
mod shift_jis_2004;

pub use ascii::{Ascii, IteratorExt as AsciiIteratorExt};
pub use jis_x_0201::{IteratorExt as JisX0201IteratorExt, JisX0201};
pub use shift_jis_1997::{IteratorExt as ShiftJis1997IteratorExt, ShiftJis1997};
pub use shift_jis_2004::{IteratorExt as ShiftJis2004IteratorExt, ShiftJis2004};
