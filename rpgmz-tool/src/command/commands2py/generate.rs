mod function_call_writer;

use self::function_call_writer::FunctionCallWriter;
use super::Command;
use super::Config;
use std::io::Write;

pub fn commands2py<W>(
    config: &Config,
    commands: &[(u16, Command)],
    mut writer: W,
) -> anyhow::Result<()>
where
    W: Write,
{
    for (indent, command) in commands.iter() {
        command2py(config, *indent, command, &mut writer)?;
    }

    Ok(())
}

fn command2py<W>(
    config: &Config,
    indent: u16,
    command: &Command,
    mut writer: W,
) -> anyhow::Result<()>
where
    W: Write,
{
    match command {
        Command::Nop => {}
        Command::ShowText {
            face_name,
            face_index,
            background,
            position_type,
            speaker_name,
            lines,
        } => {
            let mut writer = FunctionCallWriter::new(&mut writer, indent, "show_text")?;
            writer.write_param("face_name", face_name)?;
            writer.write_param("face_index", face_index)?;
            writer.write_param("background", background)?;
            writer.write_param("position_type", position_type)?;
            writer.write_param("speaker_name", speaker_name)?;
            writer.write_param("lines", lines)?;
            writer.finish()?;
        }
        Command::Comment { lines } => {
            for line in lines.iter() {
                write_indent(&mut writer, indent)?;
                writeln!(&mut writer, "# {line}")?;
            }
        }
        Command::CommonEvent { id } => {
            let name = config.get_common_event_name(*id);
            FunctionCallWriter::new(&mut writer, indent, &name)?.finish()?;
        }
        Command::FadeinScreen => {
            FunctionCallWriter::new(&mut writer, indent, "fadein_screen")?.finish()?;
        }
        Command::Unknown { code, parameters } => {
            write_indent(&mut writer, indent)?;
            writeln!(
                &mut writer,
                "# Unknown Command Code {code:?}, parameters: {parameters:?}"
            )?;
        }
    }

    Ok(())
}

fn write_indent<W>(mut writer: W, indent: u16) -> std::io::Result<()>
where
    W: Write,
{
    for _ in 0..indent {
        write!(writer, "\t")?;
    }

    Ok(())
}

fn escape_string(input: &str) -> String {
    input.replace('\'', "\\'")
}
