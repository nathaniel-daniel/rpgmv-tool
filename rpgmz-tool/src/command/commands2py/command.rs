mod code;
mod control_variables;
mod param_reader;
mod show_text;

use self::code::CommandCode;
pub use self::control_variables::ControlVariablesValue;
pub use self::control_variables::ControlVariablesValueGameData;
pub use self::control_variables::OperateVariableOperation;
use self::param_reader::ParamReader;
use anyhow::Context;

/// A command
#[derive(Debug)]
pub enum Command {
    Nop,
    ShowText {
        face_name: String,
        face_index: u32,
        background: u32,
        position_type: u32,
        speaker_name: Option<String>,
        lines: Vec<String>,
    },
    Comment {
        lines: Vec<String>,
    },
    CommonEvent {
        id: u32,
    },
    ControlVariables {
        start_variable_id: u32,
        end_variable_id: u32,
        operation: OperateVariableOperation,
        value: ControlVariablesValue,
    },
    FadeinScreen,
    Unknown {
        code: CommandCode,
        parameters: Vec<serde_json::Value>,
    },
}

impl Command {
    fn parse_nop(event_command: &rpgmz_types::EventCommand) -> anyhow::Result<Self> {
        ParamReader::new(event_command).ensure_len_is(0)?;
        Ok(Self::Nop)
    }

    fn parse_comment(event_command: &rpgmz_types::EventCommand) -> anyhow::Result<Self> {
        let reader = ParamReader::new(event_command);
        reader.ensure_len_is(1)?;

        let line = reader.read_at(0, "line")?;

        Ok(Self::Comment { lines: vec![line] })
    }

    fn parse_common_event(event_command: &rpgmz_types::EventCommand) -> anyhow::Result<Self> {
        let reader = ParamReader::new(event_command);
        reader.ensure_len_is(1)?;

        let id = reader.read_at(0, "id")?;

        Ok(Self::CommonEvent { id })
    }

    fn parse_fadein_screen(event_command: &rpgmz_types::EventCommand) -> anyhow::Result<Self> {
        ParamReader::new(event_command).ensure_len_is(0)?;
        Ok(Self::FadeinScreen)
    }
}

pub fn parse_event_command_list(
    list: &[rpgmz_types::EventCommand],
) -> anyhow::Result<Vec<(u16, Command)>> {
    let mut ret = Vec::with_capacity(list.len());

    // let mut move_command_index = 0;
    for event_command in list.iter() {
        let command_code = CommandCode(event_command.code);

        let last_command = ret.last_mut().map(|(_code, command)| command);
        let command = match (last_command, command_code) {
            (Some(Command::ShowText { lines, .. }), CommandCode::TEXT_DATA) => {
                let reader = ParamReader::new(event_command);
                reader.ensure_len_is(1)?;

                let line = reader.read_at(0, "line")?;

                lines.push(line);

                continue;
            }
            (Some(Command::Comment { lines }), CommandCode::COMMENT_EXTRA) => {
                let reader = ParamReader::new(event_command);
                reader.ensure_len_is(1)?;

                let line = reader.read_at(0, "line")?;

                lines.push(line);

                continue;
            }
            (_, CommandCode::NOP) => {
                Command::parse_nop(event_command).context("failed to parse NOP command")?
            }
            (_, CommandCode::SHOW_TEXT) => Command::parse_show_text(event_command)
                .context("failed to parse SHOW_TEXT command")?,
            (_, CommandCode::COMMENT) => {
                Command::parse_comment(event_command).context("failed to parse COMMENT command")?
            }
            (_, CommandCode::COMMON_EVENT) => Command::parse_common_event(event_command)
                .context("failed to parse COMMON_EVENT command")?,
            (_, CommandCode::CONTROL_VARIABLES) => Command::parse_control_variables(event_command)
                .context("failed to parse CONTROL_VARIABLES command")?,
            (_, CommandCode::FADEIN_SCREEN) => Command::parse_fadein_screen(event_command)
                .context("failed to parse FADEIN_SCREEN command")?,
            (_, _) => Command::Unknown {
                code: command_code,
                parameters: event_command.parameters.clone(),
            },
        };

        ret.push((event_command.indent, command));
    }

    Ok(ret)
}
