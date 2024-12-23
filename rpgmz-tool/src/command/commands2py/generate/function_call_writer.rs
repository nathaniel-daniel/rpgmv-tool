use super::escape_string;
use super::stringify_bool;
use super::write_indent;
use anyhow::bail;
use anyhow::Context;
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

impl FunctionParamValue for i64 {
    fn write_param_value(&self, writer: &mut dyn Write, _indent: u16) -> anyhow::Result<()> {
        write!(writer, "{}", self)?;

        Ok(())
    }
}

impl FunctionParamValue for u32 {
    fn write_param_value(&self, writer: &mut dyn Write, _indent: u16) -> anyhow::Result<()> {
        write!(writer, "{}", self)?;

        Ok(())
    }
}

impl FunctionParamValue for i32 {
    fn write_param_value(&self, writer: &mut dyn Write, _indent: u16) -> anyhow::Result<()> {
        write!(writer, "{}", self)?;

        Ok(())
    }
}

impl FunctionParamValue for u8 {
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

impl<T> FunctionParamValue for Option<T>
where
    T: FunctionParamValue,
{
    fn write_param_value(&self, writer: &mut dyn Write, indent: u16) -> anyhow::Result<()> {
        match self.as_ref() {
            Some(value) => value.write_param_value(writer, indent),
            None => {
                write!(writer, "None")?;
                Ok(())
            }
        }
    }
}

impl FunctionParamValue for rpgmz_types::MoveRoute {
    fn write_param_value(&self, mut writer: &mut dyn Write, indent: u16) -> anyhow::Result<()> {
        let repeat = stringify_bool(self.repeat);
        let skippable = stringify_bool(self.skippable);
        let wait = stringify_bool(self.wait);

        writeln!(writer, "MoveRoute(")?;

        write_indent(&mut writer, indent + 1)?;
        writeln!(writer, "repeat={repeat},")?;

        write_indent(&mut writer, indent + 1)?;
        writeln!(writer, "skippable={skippable},")?;

        write_indent(&mut writer, indent + 1)?;
        writeln!(&mut writer, "wait={wait},")?;

        write_indent(&mut writer, indent + 1)?;
        writeln!(writer, "list=[")?;

        for command in self.list.iter() {
            let command_indent = command
                .indent
                .map(|indent| indent.to_string())
                .unwrap_or_else(|| "None".to_string());

            write_indent(&mut writer, indent + 2)?;
            writeln!(writer, "MoveCommand(")?;

            write_indent(&mut writer, indent + 3)?;
            writeln!(writer, "code={},", command.code)?;

            write_indent(&mut writer, indent + 3)?;
            writeln!(writer, "indent={command_indent},")?;

            match command.parameters.as_ref() {
                Some(parameters) => {
                    write_indent(&mut writer, indent + 3)?;
                    writeln!(writer, "parameters=[")?;

                    for parameter in parameters {
                        match parameter {
                            serde_json::Value::Number(number) if number.is_i64() => {
                                let value = number.as_i64().context("value is not an i64")?;

                                write_indent(&mut writer, indent + 4)?;
                                writeln!(writer, "{value},")?;
                            }
                            serde_json::Value::String(value) => {
                                let value = escape_string(value);
                                write_indent(&mut writer, indent + 4)?;
                                writeln!(writer, "'{value}',")?;
                            }
                            serde_json::Value::Object(object) => {
                                write_indent(&mut writer, indent + 4)?;
                                writeln!(writer, "{{")?;

                                for (key, value) in object.iter() {
                                    write_indent(&mut writer, indent + 5)?;
                                    writeln!(writer, "'{key}': {value},")?;
                                }

                                write_indent(&mut writer, indent + 4)?;
                                writeln!(writer, "}},")?;
                            }
                            _ => {
                                bail!("cannot write move route parameter \"{parameter:?}\"")
                            }
                        }
                    }

                    write_indent(&mut writer, indent + 3)?;
                    writeln!(writer, "],")?;
                }
                None => {
                    write_indent(&mut writer, indent + 3)?;
                    writeln!(writer, "parameters=None,")?;
                }
            }

            write_indent(&mut writer, indent + 2)?;
            writeln!(writer, "),")?;
        }

        write_indent(&mut writer, indent + 1)?;
        writeln!(writer, "]")?;

        write_indent(&mut writer, indent)?;
        write!(writer, ")")?;

        Ok(())
    }
}

impl FunctionParamValue for rpgmz_types::AudioFile {
    fn write_param_value(&self, mut writer: &mut dyn Write, indent: u16) -> anyhow::Result<()> {
        let audio_name = escape_string(&self.name);

        writeln!(writer, "AudioFile(")?;

        write_indent(&mut writer, indent + 1)?;
        writeln!(writer, "name='{audio_name}',")?;

        write_indent(&mut writer, indent + 1)?;
        writeln!(writer, "pan={},", self.pan)?;

        write_indent(&mut writer, indent + 1)?;
        writeln!(writer, "pitch={},", self.pitch)?;

        write_indent(&mut writer, indent + 1)?;
        writeln!(writer, "volume={},", self.volume)?;

        write_indent(&mut writer, indent)?;
        write!(writer, ")")?;

        Ok(())
    }
}

pub struct Ident<'a>(pub &'a str);

impl FunctionParamValue for Ident<'_> {
    fn write_param_value(&self, writer: &mut dyn Write, _indent: u16) -> anyhow::Result<()> {
        write!(writer, "{}", self.0)?;
        Ok(())
    }
}
