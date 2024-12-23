mod function_call_writer;

use self::function_call_writer::FunctionCallWriter;
use super::Command;
use super::Config;
use super::ControlVariablesValue;
use super::ControlVariablesValueGameData;
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
        Command::ControlVariables {
            start_variable_id,
            end_variable_id,
            operation,
            value,
        } => {
            let operation = operation.as_str();
            let value = match value {
                ControlVariablesValue::Constant { value } => value.to_string(),
                ControlVariablesValue::Variable { id } => config.get_variable_name(*id),
                ControlVariablesValue::Random { start, stop } => {
                    format!("random.randrange(start={start}, stop={stop})")
                }
                ControlVariablesValue::GameData(game_data) => match game_data {
                    ControlVariablesValueGameData::NumItems { item_id } => {
                        let name = config.get_item_name(*item_id);

                        format!("game_party.get_num_items(item={name})")
                    }
                    ControlVariablesValueGameData::ActorLevel { actor_id } => {
                        let name = config.get_actor_name(*actor_id);
                        format!("{name}.level")
                    }
                    ControlVariablesValueGameData::ActorHp { actor_id } => {
                        let name = config.get_actor_name(*actor_id);
                        format!("{name}.hp")
                    }
                    ControlVariablesValueGameData::ActorMp { actor_id } => {
                        let name = config.get_actor_name(*actor_id);
                        format!("{name}.mp")
                    }
                    ControlVariablesValueGameData::CharacterMapX { character_id } => {
                        format!("game.get_character(id={character_id}).map_x")
                    }
                    ControlVariablesValueGameData::CharacterMapY { character_id } => {
                        format!("game.get_character(id={character_id}).map_y")
                    }
                    ControlVariablesValueGameData::MapId => "game_map.map_id()".to_string(),
                    ControlVariablesValueGameData::Gold => "game_party.gold".to_string(),
                    ControlVariablesValueGameData::Steps => "game_party.steps".to_string(),
                },
            };
            for variable_id in *start_variable_id..(*end_variable_id + 1) {
                let name = config.get_variable_name(variable_id);

                write_indent(&mut writer, indent)?;
                writeln!(&mut writer, "{name} {operation} {value}")?;
            }
        }
        Command::ControlSelfSwitch { key, value } => {
            let value = stringify_bool(*value);

            write_indent(&mut writer, indent)?;
            writeln!(&mut writer, "game_self_switches['{key}'] = {value}")?;
        }
        Command::SetMovementRoute {
            character_id,
            route,
        } => {
            let mut writer = FunctionCallWriter::new(&mut writer, indent, "set_movement_route")?;
            writer.write_param("character_id", character_id)?;
            writer.write_param("route", route)?;
            writer.finish()?;
        }
        Command::FadeinScreen => {
            FunctionCallWriter::new(&mut writer, indent, "fadein_screen")?.finish()?;
        }
        Command::ErasePicture { picture_id } => {
            let mut writer = FunctionCallWriter::new(&mut writer, indent, "erase_picture")?;
            writer.set_multiline(false);
            writer.write_param("picture_id", picture_id)?;
            writer.finish()?;
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

fn stringify_bool(b: bool) -> &'static str {
    match b {
        true => "True",
        false => "False",
    }
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
