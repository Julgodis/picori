use std::io::{BufWriter, Write};

use crate::Error;

pub trait GenTableValue {
    fn gen_table_value(&self) -> String;
}

impl GenTableValue for u8 {
    fn gen_table_value(&self) -> String { format!("{}", self) }
}

impl GenTableValue for u32 {
    fn gen_table_value(&self) -> String { format!("{}", self) }
}

impl GenTableValue for usize {
    fn gen_table_value(&self) -> String { format!("{}", self) }
}

impl<T0, T1> GenTableValue for (T0, T1)
where
    T0: GenTableValue,
    T1: GenTableValue,
{
    fn gen_table_value(&self) -> String {
        format!(
            "({},{})",
            self.0.gen_table_value(),
            self.1.gen_table_value()
        )
    }
}

impl<T0, T1, T2> GenTableValue for (T0, T1, T2)
where
    T0: GenTableValue,
    T1: GenTableValue,
    T2: GenTableValue,
{
    fn gen_table_value(&self) -> String {
        format!(
            "({},{},{})",
            self.0.gen_table_value(),
            self.1.gen_table_value(),
            self.2.gen_table_value(),
        )
    }
}

pub trait GenTable {
    fn gen_table<W>(&self, name: String, buffer: &mut BufWriter<W>) -> Result<(), Error>
    where
        W: Write;
}

impl<T> GenTable for Vec<T>
where
    T: GenTableValue,
{
    fn gen_table<W>(&self, name: String, buffer: &mut BufWriter<W>) -> Result<(), Error>
    where
        W: Write,
    {
        write!(
            buffer,
            "static {}:[{};{}]=[",
            name,
            std::any::type_name::<T>(),
            self.len()
        )?;
        for (i, value) in self.iter().enumerate() {
            write!(buffer, "{},", value.gen_table_value())?;
            if i % 32 == 31 {
                writeln!(buffer)?;
            }
        }
        writeln!(buffer, "];")?;
        Ok(())
    }
}
