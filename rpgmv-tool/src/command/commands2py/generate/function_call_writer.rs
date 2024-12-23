use super::escape_string;
use super::write_indent;
use std::io::Write;

#[derive(Debug)]
pub struct FunctionCallWriter<W> {
    writer: W,
    indent: u16,

    has_params: bool,
    multiline: bool,
}

impl<W> FunctionCallWriter<W>
where
    W: Write,
{
    pub fn new(mut writer: W, indent: u16, name: &str) -> anyhow::Result<Self> {
        write_indent(&mut writer, indent)?;
        write!(writer, "{name}(")?;

        Ok(Self {
            writer,
            indent,
            has_params: false,
            multiline: true,
        })
    }

    pub fn set_multiline(&mut self, multiline: bool) {
        self.multiline = multiline;
    }

    pub fn write_param<T>(&mut self, name: &str, param: &T) -> anyhow::Result<()>
    where
        T: FunctionParamValue,
    {
        if self.has_params {
            write!(self.writer, ",")?;
        }
        if self.multiline {
            writeln!(self.writer)?;
            write_indent(&mut self.writer, self.indent + 1)?;
        }
        write!(self.writer, "{name}=")?;
        param.write_param_value(&mut self.writer, self.indent + 1)?;

        self.has_params = true;

        Ok(())
    }

    pub fn finish(&mut self) -> anyhow::Result<()> {
        if self.has_params && self.multiline {
            writeln!(self.writer, ",")?;
            write_indent(&mut self.writer, self.indent)?;
        }

        writeln!(&mut self.writer, ")")?;

        Ok(())
    }
}

pub trait FunctionParamValue {
    fn write_param_value(&self, writer: &mut dyn Write, indent: u16) -> anyhow::Result<()>;
}

impl FunctionParamValue for str {
    fn write_param_value(&self, writer: &mut dyn Write, _indent: u16) -> anyhow::Result<()> {
        let value = escape_string(self);
        write!(writer, "'{value}'")?;
        Ok(())
    }
}

impl FunctionParamValue for String {
    fn write_param_value(&self, writer: &mut dyn Write, indent: u16) -> anyhow::Result<()> {
        self.as_str().write_param_value(writer, indent)
    }
}

impl FunctionParamValue for u32 {
    fn write_param_value(&self, writer: &mut dyn Write, _indent: u16) -> anyhow::Result<()> {
        write!(writer, "{}", self)?;

        Ok(())
    }
}

impl<T> FunctionParamValue for &[T]
where
    T: FunctionParamValue,
{
    fn write_param_value(&self, mut writer: &mut dyn Write, indent: u16) -> anyhow::Result<()> {
        writeln!(writer, "[")?;

        for entry in self.iter() {
            write_indent(&mut writer, indent + 1)?;
            entry.write_param_value(writer, indent + 1)?;
            writeln!(writer, ",")?;
        }

        write_indent(&mut writer, indent)?;
        write!(writer, "]")?;

        Ok(())
    }
}

impl<T> FunctionParamValue for Vec<T>
where
    T: FunctionParamValue,
{
    fn write_param_value(&self, writer: &mut dyn Write, indent: u16) -> anyhow::Result<()> {
        self.as_slice().write_param_value(writer, indent)
    }
}
