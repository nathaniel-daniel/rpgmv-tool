mod battle_processing;
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
    Loop,
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
    ChangeItems {
        item_id: u32,
        is_add: bool,
        value: MaybeRef<u32>,
    },
    ChangeArmors {
        armor_id: u32,
        is_add: bool,
        value: MaybeRef<u32>,
        include_equipped: bool,
    },
    ChangePartyMember {
        actor_id: u32,
        is_add: bool,
        initialize: bool,
    },
    ChangeSaveAccess {
        disable: bool,
    },
    SetEventLocation {
        character_id: i32,
        x: MaybeRef<u32>,
        y: MaybeRef<u32>,
        direction: Option<u8>,
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
    ShowBalloonIcon {
        character_id: i32,
        balloon_id: u32,
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
    FadeoutBgm {
        duration: u32,
    },
    PlaySe {
        audio: rpgmz_types::AudioFile,
    },
    BattleProcessing {
        troop_id: Option<MaybeRef<u32>>,
        can_escape: bool,
        can_lose: bool,
    },
    NameInputProcessing {
        actor_id: u32,
        max_len: u32,
    },
    ChangeHp {
        actor_id: MaybeRef<u32>,
        is_add: bool,
        value: MaybeRef<u32>,
        allow_death: bool,
    },
    ChangeMp {
        actor_id: MaybeRef<u32>,
        is_add: bool,
        value: MaybeRef<u32>,
    },
    GameOver,
    ReturnToTitleScreen,
    // We create these names based on how they are annotated in the RPGMaker code.
    // We want to follow this naming.
    #[expect(clippy::enum_variant_names)]
    PluginCommand {
        plugin_name: String,
        command_name: String,
        comment: String,
        args: serde_json::Value,
    },
    When {
        choice_index: u32,
        choice_name: String,
    },
    WhenEnd,
    Else,
    ConditionalBranchEnd,
    RepeatAbove,
    IfWin,
    IfEscape,
    IfLose,
    BattleResultEnd,
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

    fn parse_loop(event_command: &rpgmz_types::EventCommand) -> anyhow::Result<Self> {
        let reader = ParamReader::new(event_command);
        reader.ensure_len_is(0)?;
        Ok(Self::Loop)
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

    fn parse_change_items(event_command: &rpgmz_types::EventCommand) -> anyhow::Result<Self> {
        let reader = ParamReader::new(event_command);
        reader.ensure_len_is(4)?;

        let item_id = reader.read_at(0, "item_id")?;
        let IntBool(is_add) = reader.read_at(1, "is_add")?;
        let IntBool(is_constant) = reader.read_at(2, "is_constant")?;
        let value = reader.read_at(3, "is_constant")?;
        let value = if is_constant {
            MaybeRef::Constant(value)
        } else {
            MaybeRef::Ref(value)
        };

        Ok(Self::ChangeItems {
            item_id,
            is_add,
            value,
        })
    }

    fn parse_change_armors(event_command: &rpgmz_types::EventCommand) -> anyhow::Result<Self> {
        let reader = ParamReader::new(event_command);
        reader.ensure_len_is(5)?;

        let armor_id = reader.read_at(0, "armor_id")?;
        let IntBool(is_add) = reader.read_at(1, "is_add")?;
        let IntBool(is_constant) = reader.read_at(2, "is_constant")?;
        let value = reader.read_at(3, "value")?;
        let value = if is_constant {
            MaybeRef::Constant(value)
        } else {
            MaybeRef::Ref(value)
        };
        let include_equipped = reader.read_at(4, "include_equipped")?;

        Ok(Command::ChangeArmors {
            armor_id,
            is_add,
            value,
            include_equipped,
        })
    }

    fn parse_change_party_member(
        event_command: &rpgmz_types::EventCommand,
    ) -> anyhow::Result<Self> {
        let reader = ParamReader::new(event_command);
        reader.ensure_len_is(3)?;

        let actor_id = reader.read_at(0, "actor_id")?;
        let IntBool(is_add) = reader.read_at(1, "is_add")?;
        let initialize = reader.read_at(2, "initialize")?;

        Ok(Self::ChangePartyMember {
            actor_id,
            is_add,
            initialize,
        })
    }

    fn parse_change_save_access(event_command: &rpgmz_types::EventCommand) -> anyhow::Result<Self> {
        let reader = ParamReader::new(event_command);
        reader.ensure_len_is(1)?;

        let IntBool(disable) = reader.read_at(0, "disable")?;

        Ok(Self::ChangeSaveAccess { disable })
    }

    fn parse_set_event_location(event_command: &rpgmz_types::EventCommand) -> anyhow::Result<Self> {
        let reader = ParamReader::new(event_command);
        reader.ensure_len_is(5)?;

        let character_id = reader.read_at(0, "character_id")?;
        let IntBool(is_constant) = reader.read_at(1, "is_constant")?;
        let x = reader.read_at(2, "x")?;
        let y = reader.read_at(3, "y")?;
        let (x, y) = if is_constant {
            (MaybeRef::Constant(x), MaybeRef::Constant(y))
        } else {
            (MaybeRef::Ref(x), MaybeRef::Ref(y))
        };
        let direction = reader.read_at(4, "y")?;
        let direction = if direction == 0 {
            None
        } else {
            Some(direction)
        };

        Ok(Self::SetEventLocation {
            character_id,
            x,
            y,
            direction,
        })
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

    fn parse_show_balloon_icon(event_command: &rpgmz_types::EventCommand) -> anyhow::Result<Self> {
        let reader = ParamReader::new(event_command);
        reader.ensure_len_is(3)?;

        let character_id = reader.read_at(0, "character_id")?;
        let balloon_id = reader.read_at(1, "balloon_id")?;
        let wait = reader.read_at(2, "wait")?;

        Ok(Self::ShowBalloonIcon {
            character_id,
            balloon_id,
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

    fn parse_fadeout_bgm(event_command: &rpgmz_types::EventCommand) -> anyhow::Result<Self> {
        let reader = ParamReader::new(event_command);
        reader.ensure_len_is(1)?;

        let duration = reader.read_at(0, "duration")?;

        Ok(Self::FadeoutBgm { duration })
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

    fn parse_change_hp(event_command: &rpgmz_types::EventCommand) -> anyhow::Result<Self> {
        let reader = ParamReader::new(event_command);
        reader.ensure_len_is(6)?;

        let IntBool(is_actor_constant) = reader.read_at(0, "is_actor_constant")?;
        let actor_id = reader.read_at(1, "actor_id")?;
        let actor_id = if is_actor_constant {
            MaybeRef::Constant(actor_id)
        } else {
            MaybeRef::Ref(actor_id)
        };
        let IntBool(is_add) = reader.read_at(2, "is_add")?;
        let IntBool(is_constant) = reader.read_at(3, "is_constant")?;
        let value = reader.read_at(4, "value")?;
        let value = if is_constant {
            MaybeRef::Constant(value)
        } else {
            MaybeRef::Ref(value)
        };
        let allow_death = reader.read_at(5, "allow_death")?;

        Ok(Self::ChangeHp {
            actor_id,
            is_add,
            value,
            allow_death,
        })
    }

    fn parse_change_mp(event_command: &rpgmz_types::EventCommand) -> anyhow::Result<Self> {
        let reader = ParamReader::new(event_command);
        reader.ensure_len_is(5)?;

        let IntBool(is_actor_constant) = reader.read_at(0, "is_actor_constant")?;
        let actor_id = reader.read_at(1, "actor_id")?;
        let actor_id = if is_actor_constant {
            MaybeRef::Constant(actor_id)
        } else {
            MaybeRef::Ref(actor_id)
        };
        let IntBool(is_add) = reader.read_at(2, "is_add")?;
        let IntBool(is_constant) = reader.read_at(3, "is_constant")?;
        let value = reader.read_at(4, "is_constant")?;
        let value = if is_constant {
            MaybeRef::Constant(value)
        } else {
            MaybeRef::Ref(value)
        };

        Ok(Self::ChangeMp {
            actor_id,
            is_add,
            value,
        })
    }

    fn parse_game_over(event_command: &rpgmz_types::EventCommand) -> anyhow::Result<Self> {
        let reader = ParamReader::new(event_command);
        reader.ensure_len_is(0)?;

        Ok(Self::GameOver)
    }

    fn parse_return_to_title_screen(
        event_command: &rpgmz_types::EventCommand,
    ) -> anyhow::Result<Self> {
        let reader = ParamReader::new(event_command);
        reader.ensure_len_is(0)?;

        Ok(Self::ReturnToTitleScreen)
    }

    fn parse_plugin_command(event_command: &rpgmz_types::EventCommand) -> anyhow::Result<Self> {
        let reader = ParamReader::new(event_command);
        reader.ensure_len_is(4)?;

        let plugin_name = reader.read_at(0, "plugin_name")?;
        let command_name = reader.read_at(1, "command_name")?;
        let comment = reader.read_at(2, "comment")?;
        let args = reader.read_at(3, "args")?;

        Ok(Self::PluginCommand {
            plugin_name,
            command_name,
            comment,
            args,
        })
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

    fn parse_repeat_above(event_command: &rpgmz_types::EventCommand) -> anyhow::Result<Self> {
        ParamReader::new(event_command).ensure_len_is(0)?;
        Ok(Self::RepeatAbove)
    }

    fn parse_if_win(event_command: &rpgmz_types::EventCommand) -> anyhow::Result<Self> {
        ParamReader::new(event_command).ensure_len_is(0)?;
        Ok(Self::IfWin)
    }

    fn parse_if_escape(event_command: &rpgmz_types::EventCommand) -> anyhow::Result<Self> {
        ParamReader::new(event_command).ensure_len_is(0)?;
        Ok(Self::IfEscape)
    }

    fn parse_if_lose(event_command: &rpgmz_types::EventCommand) -> anyhow::Result<Self> {
        ParamReader::new(event_command).ensure_len_is(0)?;
        Ok(Self::IfLose)
    }

    fn parse_battle_result_end(event_command: &rpgmz_types::EventCommand) -> anyhow::Result<Self> {
        ParamReader::new(event_command).ensure_len_is(0)?;
        Ok(Self::BattleResultEnd)
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
            (Some(Command::PluginCommand { .. }), CommandCode::PLUGIN_COMMAND_EXTRA) => {
                // I'm not sure about the plugin command arg format,
                // so I don't do checks here.
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
            (_, CommandCode::LOOP) => {
                Command::parse_loop(event_command).context("failed to parse LOOP command")?
            }
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
            (_, CommandCode::CHANGE_ITEMS) => Command::parse_change_items(event_command)
                .context("failed to parse CHANGE_ITEMS command")?,
            (_, CommandCode::CHANGE_ARMORS) => Command::parse_change_armors(event_command)
                .context("failed to parse CHANGE_ARMORS command")?,
            (_, CommandCode::CHANGE_PARTY_MEMBER) => {
                Command::parse_change_party_member(event_command)
                    .context("failed to parse CHANGE_PARTY_MEMBER command")?
            }
            (_, CommandCode::CHANGE_SAVE_ACCESS) => {
                Command::parse_change_save_access(event_command)
                    .context("failed to parse CHANGE_SAVE_ACCESS command")?
            }
            (_, CommandCode::SET_EVENT_LOCATION) => {
                Command::parse_set_event_location(event_command)
                    .context("failed to parse SET_EVENT_LOCATION command")?
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
            (_, CommandCode::SHOW_BALLOON_ICON) => Command::parse_show_balloon_icon(event_command)
                .context("failed to parse SHOW_BALLOON_ICON command")?,
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
            (_, CommandCode::FADEOUT_BGM) => Command::parse_fadeout_bgm(event_command)
                .context("failed to parse FADEOUT_BGM command")?,
            (_, CommandCode::PLAY_SE) => {
                Command::parse_play_se(event_command).context("failed to parse PLAY_SE command")?
            }
            (_, CommandCode::BATTLE_PROCESSING) => Command::parse_battle_processing(event_command)
                .context("failed to parse BATTLE_PROCESSING command")?,
            (_, CommandCode::NAME_INPUT_PROCESSING) => {
                Command::parse_name_input_processing(event_command)
                    .context("failed to parse NAME_INPUT_PROCESSING command")?
            }
            (_, CommandCode::CHANGE_HP) => Command::parse_change_hp(event_command)
                .context("failed to parse CHANGE_HP command")?,
            (_, CommandCode::CHANGE_MP) => Command::parse_change_mp(event_command)
                .context("failed to parse CHANGE_MP command")?,
            (_, CommandCode::GAME_OVER) => Command::parse_game_over(event_command)
                .context("failed to parse GAME_OVER command")?,
            (_, CommandCode::RETURN_TO_TITLE_SCREEN) => {
                Command::parse_return_to_title_screen(event_command)
                    .context("failed to parse RETURN_TO_TITLE_SCREEN command")?
            }
            (_, CommandCode::PLUGIN_COMMAND) => Command::parse_plugin_command(event_command)
                .context("failed to parse PLUGIN_COMMAND command")?,
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
            (_, CommandCode::REPEAT_ABOVE) => Command::parse_repeat_above(event_command)
                .context("failed to parse REPEAT_ABOVE command")?,
            (_, CommandCode::IF_WIN) => {
                Command::parse_if_win(event_command).context("failed to parse IF_WIN command")?
            }
            (_, CommandCode::IF_ESCAPE) => Command::parse_if_escape(event_command)
                .context("failed to parse IF_ESCAPE command")?,
            (_, CommandCode::IF_LOSE) => {
                Command::parse_if_lose(event_command).context("failed to parse IF_LOSE command")?
            }
            (_, CommandCode::BATTLE_RESULT_END) => Command::parse_battle_result_end(event_command)
                .context("failed to parse BATTLE_RESULT_END command")?,
            (_, _) => Command::Unknown {
                code: command_code,
                parameters: event_command.parameters.clone(),
            },
        };

        ret.push((event_command.indent, command));
    }

    Ok(ret)
}
