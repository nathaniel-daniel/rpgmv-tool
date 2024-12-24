mod code;
mod conditional_branch;
mod control_variables;
mod param_reader;
mod show_text;

use self::code::CommandCode;
pub use self::conditional_branch::ConditionalBranchCommand;
pub use self::control_variables::ControlVariablesValue;
pub use self::control_variables::ControlVariablesValueGameData;
pub use self::control_variables::OperateVariableOperation;
use self::param_reader::IntBool;
use self::param_reader::ParamReader;
use anyhow::ensure;
use anyhow::Context;

#[derive(Debug, Copy, Clone, Hash)]
pub enum MaybeRef<T> {
    Constant(T),
    Ref(u32),
}

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
    ShowChoices {
        choices: Vec<String>,
        cancel_type: i32,
        default_type: i64,
        position_type: u32,
        background: u32,
    },
    Comment {
        lines: Vec<String>,
    },
    ConditionalBranch(ConditionalBranchCommand),
    CommonEvent {
        id: u32,
    },
    Label {
        name: String,
    },
    JumpToLabel {
        name: String,
    },
    ControlSwitches {
        start_id: u32,
        end_id: u32,
        value: bool,
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
    TransferPlayer {
        map_id: MaybeRef<u32>,
        x: MaybeRef<u32>,
        y: MaybeRef<u32>,
        direction: u8,
        fade_type: u8,
    },
    SetMovementRoute {
        character_id: i32,
        route: rpgmz_types::MoveRoute,
    },
    ShowAnimation {
        character_id: i32,
        animation_id: u32,
        wait: bool,
    },
    FadeoutScreen,
    FadeinScreen,
    ShakeScreen {
        power: u32,
        speed: u32,
        duration: u32,
        wait: bool,
    },
    Wait {
        duration: u32,
    },
    ErasePicture {
        picture_id: u32,
    },
    PlayBgm {
        audio: rpgmz_types::AudioFile,
    },
    PlaySe {
        audio: rpgmz_types::AudioFile,
    },
    NameInputProcessing {
        actor_id: u32,
        max_len: u32,
    },
    When {
        choice_index: u32,
        choice_name: String,
    },
    WhenEnd,
    Else,
    ConditionalBranchEnd,
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

    fn parse_show_choices(event_command: &rpgmz_types::EventCommand) -> anyhow::Result<Self> {
        let reader = ParamReader::new(event_command);
        reader.ensure_len_is(5)?;

        let choices = reader.read_at(0, "choices")?;
        let cancel_type = reader.read_at(1, "cancel_type")?;
        let default_type = reader.read_at(2, "default_type")?;
        let position_type = reader.read_at(3, "position_type")?;
        let background = reader.read_at(4, "background")?;

        Ok(Command::ShowChoices {
            choices,
            cancel_type,
            default_type,
            position_type,
            background,
        })
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

    fn parse_label(event_command: &rpgmz_types::EventCommand) -> anyhow::Result<Self> {
        let reader = ParamReader::new(event_command);
        reader.ensure_len_is(1)?;

        let name = reader.read_at(0, "name")?;

        Ok(Self::Label { name })
    }

    fn parse_jump_to_label(event_command: &rpgmz_types::EventCommand) -> anyhow::Result<Self> {
        let reader = ParamReader::new(event_command);
        reader.ensure_len_is(1)?;

        let name = reader.read_at(0, "name")?;

        Ok(Self::JumpToLabel { name })
    }

    fn parse_control_switches(event_command: &rpgmz_types::EventCommand) -> anyhow::Result<Self> {
        let reader = ParamReader::new(event_command);
        reader.ensure_len_is(3)?;

        let start_id = reader.read_at(0, "start_id")?;
        let end_id = reader.read_at(1, "end_id")?;
        let IntBool(value) = reader.read_at(2, "value")?;

        Ok(Self::ControlSwitches {
            start_id,
            end_id,
            value,
        })
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

    fn parse_transfer_player(event_command: &rpgmz_types::EventCommand) -> anyhow::Result<Self> {
        let reader = ParamReader::new(event_command);
        reader.ensure_len_is(6)?;

        let IntBool(is_constant) = reader.read_at(0, "is_constant")?;
        let map_id = reader.read_at(1, "map_id")?;
        let x = reader.read_at(2, "x")?;
        let y = reader.read_at(3, "y")?;
        let direction = reader.read_at(4, "direction")?;
        let fade_type = reader.read_at(5, "fade_type")?;

        let (map_id, x, y) = if is_constant {
            (
                MaybeRef::Constant(map_id),
                MaybeRef::Constant(x),
                MaybeRef::Constant(y),
            )
        } else {
            (MaybeRef::Ref(map_id), MaybeRef::Ref(x), MaybeRef::Ref(y))
        };

        Ok(Command::TransferPlayer {
            map_id,
            x,
            y,
            direction,
            fade_type,
        })
    }

    fn parse_fadeout_screen(event_command: &rpgmz_types::EventCommand) -> anyhow::Result<Self> {
        ParamReader::new(event_command).ensure_len_is(0)?;
        Ok(Self::FadeoutScreen)
    }

    fn parse_fadein_screen(event_command: &rpgmz_types::EventCommand) -> anyhow::Result<Self> {
        ParamReader::new(event_command).ensure_len_is(0)?;
        Ok(Self::FadeinScreen)
    }

    fn parse_shake_screen(event_command: &rpgmz_types::EventCommand) -> anyhow::Result<Self> {
        let reader = ParamReader::new(event_command);
        reader.ensure_len_is(4)?;

        let power = reader.read_at(0, "power")?;
        let speed = reader.read_at(1, "speed")?;
        let duration = reader.read_at(2, "duration")?;
        let wait = reader.read_at(3, "wait")?;

        Ok(Self::ShakeScreen {
            power,
            speed,
            duration,
            wait,
        })
    }

    fn parse_wait(event_command: &rpgmz_types::EventCommand) -> anyhow::Result<Self> {
        let reader = ParamReader::new(event_command);
        reader.ensure_len_is(1)?;

        let duration = reader.read_at(0, "duration")?;

        Ok(Self::Wait { duration })
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

    fn parse_show_animation(event_command: &rpgmz_types::EventCommand) -> anyhow::Result<Self> {
        let reader = ParamReader::new(event_command);
        reader.ensure_len_is(3)?;

        let character_id = reader.read_at(0, "character_id")?;
        let animation_id = reader.read_at(1, "animation_id")?;
        let wait = reader.read_at(2, "animation_id")?;

        Ok(Self::ShowAnimation {
            character_id,
            animation_id,
            wait,
        })
    }

    fn parse_erase_picture(event_command: &rpgmz_types::EventCommand) -> anyhow::Result<Self> {
        let reader = ParamReader::new(event_command);
        reader.ensure_len_is(1)?;

        let picture_id = reader.read_at(0, "picture_id")?;

        Ok(Self::ErasePicture { picture_id })
    }

    fn parse_play_bgm(event_command: &rpgmz_types::EventCommand) -> anyhow::Result<Self> {
        let reader = ParamReader::new(event_command);
        reader.ensure_len_is(1)?;

        let audio = reader.read_at(0, "audio")?;

        Ok(Self::PlayBgm { audio })
    }

    fn parse_play_se(event_command: &rpgmz_types::EventCommand) -> anyhow::Result<Self> {
        let reader = ParamReader::new(event_command);
        reader.ensure_len_is(1)?;

        let audio = reader.read_at(0, "audio")?;

        Ok(Self::PlaySe { audio })
    }

    fn parse_name_input_processing(
        event_command: &rpgmz_types::EventCommand,
    ) -> anyhow::Result<Self> {
        let reader = ParamReader::new(event_command);
        reader.ensure_len_is(2)?;

        let actor_id = reader.read_at(0, "actor_id")?;
        let max_len = reader.read_at(1, "max_len")?;

        Ok(Self::NameInputProcessing { actor_id, max_len })
    }

    fn parse_when(event_command: &rpgmz_types::EventCommand) -> anyhow::Result<Self> {
        let reader = ParamReader::new(event_command);
        reader.ensure_len_is(2)?;

        let choice_index = reader.read_at(0, "choice_index")?;
        let choice_name = reader.read_at(1, "choice_name")?;

        Ok(Self::When {
            choice_index,
            choice_name,
        })
    }

    fn parse_when_end(event_command: &rpgmz_types::EventCommand) -> anyhow::Result<Self> {
        ParamReader::new(event_command).ensure_len_is(0)?;
        Ok(Self::WhenEnd)
    }

    fn parse_else(event_command: &rpgmz_types::EventCommand) -> anyhow::Result<Self> {
        ParamReader::new(event_command).ensure_len_is(0)?;

        Ok(Command::Else)
    }

    fn parse_conditional_branch_end(
        event_command: &rpgmz_types::EventCommand,
    ) -> anyhow::Result<Self> {
        ParamReader::new(event_command).ensure_len_is(0)?;
        Ok(Self::ConditionalBranchEnd)
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
            (_, CommandCode::SHOW_CHOICES) => Command::parse_show_choices(event_command)
                .context("failed to parse SHOW_CHOICES command")?,
            (_, CommandCode::COMMENT) => {
                Command::parse_comment(event_command).context("failed to parse COMMENT command")?
            }
            (_, CommandCode::CONDITONAL_BRANCH) => Command::parse_conditional_branch(event_command)
                .context("failed to parse CONDITONAL_BRANCH command")?,
            (_, CommandCode::COMMON_EVENT) => Command::parse_common_event(event_command)
                .context("failed to parse COMMON_EVENT command")?,
            (_, CommandCode::LABEL) => {
                Command::parse_label(event_command).context("failed to parse LABEL command")?
            }
            (_, CommandCode::JUMP_TO_LABEL) => Command::parse_jump_to_label(event_command)
                .context("failed to parse JUMP_TO_LABEL command")?,
            (_, CommandCode::CONTROL_SWITCHES) => Command::parse_control_switches(event_command)
                .context("failed to parse CONTROL_SWITCHES command")?,
            (_, CommandCode::CONTROL_VARIABLES) => Command::parse_control_variables(event_command)
                .context("failed to parse CONTROL_VARIABLES command")?,
            (_, CommandCode::CONTROL_SELF_SWITCH) => {
                Command::parse_control_self_switch(event_command)
                    .context("failed to parse CONTROL_SELF_SWITCH command")?
            }
            (_, CommandCode::TRANSFER_PLAYER) => Command::parse_transfer_player(event_command)
                .context("failed to parse TRANSFER_PLAYER command")?,
            (_, CommandCode::SET_MOVEMENT_ROUTE) => {
                move_command_index = 0;

                Command::parse_set_movement_route(event_command)
                    .context("failed to parse SET_MOVEMENT_ROUTE command")?
            }
            (_, CommandCode::SHOW_ANIMATION) => Command::parse_show_animation(event_command)
                .context("failed to parse SHOW_ANIMATION command")?,
            (_, CommandCode::FADEOUT_SCREEN) => Command::parse_fadeout_screen(event_command)
                .context("failed to parse FADEOUT_SCREEN command")?,
            (_, CommandCode::FADEIN_SCREEN) => Command::parse_fadein_screen(event_command)
                .context("failed to parse FADEIN_SCREEN command")?,
            (_, CommandCode::SHAKE_SCREEN) => Command::parse_shake_screen(event_command)
                .context("failed to parse SHAKE_SCREEN command")?,
            (_, CommandCode::WAIT) => {
                Command::parse_wait(event_command).context("failed to parse WAIT command")?
            }
            (_, CommandCode::ERASE_PICTURE) => Command::parse_erase_picture(event_command)
                .context("failed to parse ERASE_PICTURE command")?,
            (_, CommandCode::PLAY_BGM) => Command::parse_play_bgm(event_command)
                .context("failed to parse PLAY_BGM command")?,
            (_, CommandCode::PLAY_SE) => {
                Command::parse_play_se(event_command).context("failed to parse PLAY_SE command")?
            }
            (_, CommandCode::NAME_INPUT_PROCESSING) => {
                Command::parse_name_input_processing(event_command)
                    .context("failed to parse NAME_INPUT_PROCESSING command")?
            }
            (_, CommandCode::WHEN) => {
                Command::parse_when(event_command).context("failed to parse WHEN command")?
            }
            (_, CommandCode::WHEN_END) => Command::parse_when_end(event_command)
                .context("failed to parse WHEN_END command")?,
            (_, CommandCode::ELSE) => {
                Command::parse_else(event_command).context("failed to parse ELSE command")?
            }
            (_, CommandCode::CONDITONAL_BRANCH_END) => {
                Command::parse_conditional_branch_end(event_command)
                    .context("failed to parse CONDITONAL_BRANCH_END command")?
            }
            (_, _) => Command::Unknown {
                code: command_code,
                parameters: event_command.parameters.clone(),
            },
        };

        ret.push((event_command.indent, command));
    }

    Ok(ret)
}
