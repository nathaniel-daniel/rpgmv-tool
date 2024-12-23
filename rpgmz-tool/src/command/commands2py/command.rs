mod code;
mod control_variables;
mod param_reader;
mod show_text;

use self::code::CommandCode;
pub use self::control_variables::ControlVariablesValue;
pub use self::control_variables::ControlVariablesValueGameData;
pub use self::control_variables::OperateVariableOperation;
use self::param_reader::IntBool;
use self::param_reader::ParamReader;
use anyhow::ensure;
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
    ControlSelfSwitch {
        key: String,
        value: bool,
    },
    FadeinScreen,
    SetMovementRoute {
        character_id: i32,
        route: rpgmz_types::MoveRoute,
    },
    ErasePicture {
        picture_id: u32,
    },
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

    fn parse_control_self_switch(
        event_command: &rpgmz_types::EventCommand,
    ) -> anyhow::Result<Self> {
        let reader = ParamReader::new(event_command);
        reader.ensure_len_is(2)?;

        let key = reader.read_at(0, "key")?;
        let value: IntBool = reader.read_at(1, "value")?;
        let value = value.0;

        Ok(Command::ControlSelfSwitch { key, value })
    }

    fn parse_fadein_screen(event_command: &rpgmz_types::EventCommand) -> anyhow::Result<Self> {
        ParamReader::new(event_command).ensure_len_is(0)?;
        Ok(Self::FadeinScreen)
    }

    fn parse_set_movement_route(event_command: &rpgmz_types::EventCommand) -> anyhow::Result<Self> {
        let reader = ParamReader::new(event_command);
        reader.ensure_len_is(2)?;

        let character_id = reader.read_at(0, "character_id")?;
        let route = reader.read_at(1, "route")?;

        Ok(Self::SetMovementRoute {
            character_id,
            route,
        })
    }

    fn parse_erase_picture(event_command: &rpgmz_types::EventCommand) -> anyhow::Result<Self> {
        let reader = ParamReader::new(event_command);
        reader.ensure_len_is(1)?;

        let picture_id = reader.read_at(0, "picture_id")?;

        Ok(Command::ErasePicture { picture_id })
    }
}

pub fn parse_event_command_list(
    list: &[rpgmz_types::EventCommand],
) -> anyhow::Result<Vec<(u16, Command)>> {
    let mut ret = Vec::with_capacity(list.len());

    let mut move_command_index = 0;
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
            (
                Some(Command::SetMovementRoute { route, .. }),
                CommandCode::SET_MOVEMENT_ROUTE_EXTRA,
            ) if move_command_index < route.list.len() => {
                let reader = ParamReader::new(event_command);
                reader.ensure_len_is(1)?;

                let command: rpgmz_types::MoveCommand = reader.read_at(0, "command")?;

                ensure!(command == route.list[move_command_index]);

                move_command_index += 1;

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
            (_, CommandCode::CONTROL_SELF_SWITCH) => {
                Command::parse_control_self_switch(event_command)
                    .context("failed to parse CONTROL_SELF_SWITCH command")?
            }
            (_, CommandCode::SET_MOVEMENT_ROUTE) => {
                move_command_index = 0;

                Command::parse_set_movement_route(event_command)
                    .context("failed to parse SET_MOVEMENT_ROUTE command")?
            }
            (_, CommandCode::FADEIN_SCREEN) => Command::parse_fadein_screen(event_command)
                .context("failed to parse FADEIN_SCREEN command")?,

            (_, CommandCode::ERASE_PICTURE) => Command::parse_erase_picture(event_command)
                .context("failed to parse ERASE_PICTURE command")?,
            (_, _) => Command::Unknown {
                code: command_code,
                parameters: event_command.parameters.clone(),
            },
        };

        ret.push((event_command.indent, command));
    }

    Ok(ret)
}
