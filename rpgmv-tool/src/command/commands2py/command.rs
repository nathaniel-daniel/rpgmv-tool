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
use anyhow::bail;
use anyhow::ensure;
use anyhow::Context;

#[derive(Debug, Copy, Clone)]
pub enum GetLocationInfoKind {
    TerrainTag,
    EventId,
}

impl GetLocationInfoKind {
    /// Get this from a u8.
    pub fn from_u8(value: u8) -> anyhow::Result<Self> {
        match value {
            0 => Ok(Self::TerrainTag),
            1 => Ok(Self::EventId),
            _ => bail!("{value} is not a valid GetLocationInfoKind"),
        }
    }
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
        lines: Vec<String>,
    },
    ShowChoices {
        choices: Vec<String>,
        cancel_type: i32,
        default_type: i64,
        position_type: u32,
        background: u32,
    },
    ShowScrollingText {
        speed: u32,
        no_fast: bool,
        lines: Vec<String>,
    },
    Comment {
        lines: Vec<String>,
    },
    ConditionalBranch(ConditionalBranchCommand),
    ExitEventProcessing,
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
    ControlTimer {
        start_seconds: Option<u32>,
    },
    ChangeGold {
        is_add: bool,
        value: MaybeRef<u32>,
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
        route: rpgmv_types::MoveRoute,
    },
    ChangeTransparency {
        set_transparent: bool,
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
    ChangePlayerFollowers {
        is_show: bool,
    },
    FadeoutScreen,
    FadeinScreen,
    TintScreen {
        tone: [i16; 4],
        duration: u32,
        wait: bool,
    },
    FlashScreen {
        color: [u8; 4],
        duration: u32,
        wait: bool,
    },
    ShakeScreen {
        power: u32,
        speed: u32,
        duration: u32,
        wait: bool,
    },
    Wait {
        duration: u32,
    },
    ShowPicture {
        picture_id: u32,
        picture_name: String,
        origin: u32,
        x: MaybeRef<i32>,
        y: MaybeRef<i32>,
        scale_x: u32,
        scale_y: u32,
        opacity: u8,
        blend_mode: u8,
    },
    ErasePicture {
        picture_id: u32,
    },
    PlayBgm {
        audio: rpgmv_types::AudioFile,
    },
    FadeoutBgm {
        duration: u32,
    },
    SaveBgm,
    ResumeBgm,
    PlayBgs {
        audio: rpgmv_types::AudioFile,
    },
    FadeoutBgs {
        duration: u32,
    },
    PlaySe {
        audio: rpgmv_types::AudioFile,
    },
    GetLocationInfo {
        variable_id: u32,
        kind: GetLocationInfoKind,
        x: MaybeRef<u32>,
        y: MaybeRef<u32>,
    },
    BattleProcessing {
        troop_id: MaybeRef<u32>,
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
    ChangeState {
        actor_id: MaybeRef<u32>,
        is_add_state: bool,
        state_id: u32,
    },
    ChangeLevel {
        actor_id: MaybeRef<u32>,
        is_add: bool,
        value: MaybeRef<u32>,
        show_level_up: bool,
    },
    ChangeSkill {
        actor_id: MaybeRef<u32>,
        is_learn_skill: bool,
        skill_id: u32,
    },
    ChangeClass {
        actor_id: u32,
        class_id: u32,
        keep_exp: bool,
    },
    ChangeActorImages {
        actor_id: u32,
        character_name: String,
        character_index: u32,
        face_name: String,
        face_index: u32,
        battler_name: String,
    },
    ForceAction {
        is_enemy: bool,
        id: u32,
        skill_id: u32,
        target_index: u32,
    },
    AbortBattle,
    ReturnToTitleScreen,
    Script {
        lines: Vec<String>,
    },
    // We create these names based on how they are annotated in the RPGMaker code.
    // We want to follow this naming.
    #[expect(clippy::enum_variant_names)]
    PluginCommand {
        params: Vec<String>,
    },
    When {
        choice_index: u32,
        choice_name: String,
    },
    WhenCancel {
        choice_index: u32,
        choice_name: Option<String>,
    },
    WhenEnd,
    Else,
    ConditionalBranchEnd,
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
    fn parse_nop(event_command: &rpgmv_types::EventCommand) -> anyhow::Result<Self> {
        ParamReader::new(event_command).ensure_len_is(0)?;
        Ok(Self::Nop)
    }

    fn parse_show_choices(event_command: &rpgmv_types::EventCommand) -> anyhow::Result<Self> {
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

    fn parse_comment(event_command: &rpgmv_types::EventCommand) -> anyhow::Result<Self> {
        let reader = ParamReader::new(event_command);
        reader.ensure_len_is(1)?;

        let line = reader.read_at(0, "line")?;

        Ok(Self::Comment { lines: vec![line] })
    }

    fn parse_common_event(event_command: &rpgmv_types::EventCommand) -> anyhow::Result<Self> {
        let reader = ParamReader::new(event_command);
        reader.ensure_len_is(1)?;

        let id = reader.read_at(0, "id")?;

        Ok(Self::CommonEvent { id })
    }

    fn parse_label(event_command: &rpgmv_types::EventCommand) -> anyhow::Result<Self> {
        let reader = ParamReader::new(event_command);
        reader.ensure_len_is(1)?;

        let name = reader.read_at(0, "name")?;

        Ok(Self::Label { name })
    }

    fn parse_jump_to_label(event_command: &rpgmv_types::EventCommand) -> anyhow::Result<Self> {
        let reader = ParamReader::new(event_command);
        reader.ensure_len_is(1)?;

        let name = reader.read_at(0, "name")?;

        Ok(Self::JumpToLabel { name })
    }

    fn parse_control_switches(event_command: &rpgmv_types::EventCommand) -> anyhow::Result<Self> {
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
        event_command: &rpgmv_types::EventCommand,
    ) -> anyhow::Result<Self> {
        let reader = ParamReader::new(event_command);
        reader.ensure_len_is(2)?;

        let key = reader.read_at(0, "key")?;
        let value: IntBool = reader.read_at(1, "value")?;
        let value = value.0;

        Ok(Self::ControlSelfSwitch { key, value })
    }

    fn parse_change_items(event_command: &rpgmv_types::EventCommand) -> anyhow::Result<Self> {
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

    fn parse_fadeout_screen(event_command: &rpgmv_types::EventCommand) -> anyhow::Result<Self> {
        ParamReader::new(event_command).ensure_len_is(0)?;
        Ok(Self::FadeoutScreen)
    }

    fn parse_fadein_screen(event_command: &rpgmv_types::EventCommand) -> anyhow::Result<Self> {
        ParamReader::new(event_command).ensure_len_is(0)?;
        Ok(Self::FadeinScreen)
    }

    fn parse_transfer_player(event_command: &rpgmv_types::EventCommand) -> anyhow::Result<Self> {
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

        Ok(Self::TransferPlayer {
            map_id,
            x,
            y,
            direction,
            fade_type,
        })
    }

    fn parse_set_movement_route(event_command: &rpgmv_types::EventCommand) -> anyhow::Result<Self> {
        let reader = ParamReader::new(event_command);
        reader.ensure_len_is(2)?;

        let character_id = reader.read_at(0, "character_id")?;
        let route = reader.read_at(1, "route")?;

        Ok(Self::SetMovementRoute {
            character_id,
            route,
        })
    }

    fn parse_show_animation(event_command: &rpgmv_types::EventCommand) -> anyhow::Result<Self> {
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

    fn parse_show_balloon_icon(event_command: &rpgmv_types::EventCommand) -> anyhow::Result<Self> {
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

    fn parse_shake_screen(event_command: &rpgmv_types::EventCommand) -> anyhow::Result<Self> {
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

    fn parse_wait(event_command: &rpgmv_types::EventCommand) -> anyhow::Result<Self> {
        let reader = ParamReader::new(event_command);
        reader.ensure_len_is(1)?;

        let duration = reader.read_at(0, "duration")?;

        Ok(Self::Wait { duration })
    }

    fn parse_play_se(event_command: &rpgmv_types::EventCommand) -> anyhow::Result<Self> {
        let reader = ParamReader::new(event_command);
        reader.ensure_len_is(1)?;

        let audio = reader.read_at(0, "audio")?;

        Ok(Self::PlaySe { audio })
    }

    fn parse_show_picture(event_command: &rpgmv_types::EventCommand) -> anyhow::Result<Self> {
        ensure!(event_command.parameters.len() == 10);
        let picture_id = event_command.parameters[0]
            .as_i64()
            .and_then(|value| u32::try_from(value).ok())
            .context("`picture_id` is not a `u32`")?;
        let picture_name = event_command.parameters[1]
            .as_str()
            .context("`picture_name` is not a string")?
            .to_string();
        let origin = event_command.parameters[2]
            .as_i64()
            .and_then(|value| u32::try_from(value).ok())
            .context("`origin` is not a `u32`")?;
        let is_constant = event_command.parameters[3]
            .as_i64()
            .and_then(|value| u8::try_from(value).ok())
            .context("`origin` is not a `u8`")?;
        ensure!(is_constant <= 1);
        let is_constant = is_constant == 0;
        let x = event_command.parameters[4]
            .as_i64()
            .context("`x` is not an `i64`")?;
        let y = event_command.parameters[5]
            .as_i64()
            .context("`y` is not an `i64`")?;
        let scale_x = event_command.parameters[6]
            .as_i64()
            .and_then(|value| u32::try_from(value).ok())
            .context("`scale_x` is not a `u32`")?;
        let scale_y = event_command.parameters[7]
            .as_i64()
            .and_then(|value| u32::try_from(value).ok())
            .context("`scale_y` is not a `u32`")?;
        let opacity = event_command.parameters[8]
            .as_i64()
            .and_then(|value| u8::try_from(value).ok())
            .context("`opacity` is not a `u8`")?;
        let blend_mode = event_command.parameters[9]
            .as_i64()
            .and_then(|value| u8::try_from(value).ok())
            .context("`opacity` is not a `u8`")?;

        let (x, y) = if is_constant {
            let x = i32::try_from(x).context("`x` is not an `i32`")?;
            let y = i32::try_from(y).context("`y` is not an `i32`")?;

            (MaybeRef::Constant(x), MaybeRef::Constant(y))
        } else {
            let x = u32::try_from(x).context("`x` is not a `u32`")?;
            let y = u32::try_from(y).context("`y` is not a `u32`")?;

            (MaybeRef::Ref(x), MaybeRef::Ref(y))
        };

        Ok(Self::ShowPicture {
            picture_id,
            picture_name,
            origin,
            x,
            y,
            scale_x,
            scale_y,
            opacity,
            blend_mode,
        })
    }

    fn parse_erase_picture(event_command: &rpgmv_types::EventCommand) -> anyhow::Result<Self> {
        let reader = ParamReader::new(event_command);
        reader.ensure_len_is(1)?;

        let picture_id = reader.read_at(0, "picture_id")?;

        Ok(Self::ErasePicture { picture_id })
    }

    fn parse_play_bgm(event_command: &rpgmv_types::EventCommand) -> anyhow::Result<Self> {
        let reader = ParamReader::new(event_command);
        reader.ensure_len_is(1)?;

        let audio = reader.read_at(0, "audio")?;

        Ok(Self::PlayBgm { audio })
    }

    fn parse_fadeout_bgm(event_command: &rpgmv_types::EventCommand) -> anyhow::Result<Self> {
        let reader = ParamReader::new(event_command);
        reader.ensure_len_is(1)?;

        let duration = reader.read_at(0, "duration")?;

        Ok(Self::FadeoutBgm { duration })
    }

    fn parse_name_input_processing(
        event_command: &rpgmv_types::EventCommand,
    ) -> anyhow::Result<Self> {
        let reader = ParamReader::new(event_command);
        reader.ensure_len_is(2)?;

        let actor_id = reader.read_at(0, "actor_id")?;
        let max_len = reader.read_at(1, "max_len")?;

        Ok(Self::NameInputProcessing { actor_id, max_len })
    }

    fn parse_change_mp(event_command: &rpgmv_types::EventCommand) -> anyhow::Result<Self> {
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

    fn parse_when(event_command: &rpgmv_types::EventCommand) -> anyhow::Result<Self> {
        let reader = ParamReader::new(event_command);
        reader.ensure_len_is(2)?;

        let choice_index = reader.read_at(0, "choice_index")?;
        let choice_name = reader.read_at(1, "choice_name")?;

        Ok(Self::When {
            choice_index,
            choice_name,
        })
    }

    fn parse_when_end(event_command: &rpgmv_types::EventCommand) -> anyhow::Result<Self> {
        ParamReader::new(event_command).ensure_len_is(0)?;
        Ok(Self::WhenEnd)
    }

    fn parse_else(event_command: &rpgmv_types::EventCommand) -> anyhow::Result<Self> {
        ParamReader::new(event_command).ensure_len_is(0)?;
        Ok(Self::Else)
    }

    fn parse_conditional_branch_end(
        event_command: &rpgmv_types::EventCommand,
    ) -> anyhow::Result<Self> {
        ParamReader::new(event_command).ensure_len_is(0)?;
        Ok(Self::ConditionalBranchEnd)
    }
}

#[derive(Debug, Copy, Clone, Hash)]
pub enum MaybeRef<T> {
    Constant(T),
    Ref(u32),
}

pub fn parse_event_command_list(
    list: &[rpgmv_types::EventCommand],
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
            (
                Some(Command::ShowScrollingText { lines, .. }),
                CommandCode::SHOW_SCROLLING_TEXT_EXTRA,
            ) => {
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

                let command: rpgmv_types::MoveCommand = reader.read_at(0, "command")?;

                ensure!(command == route.list[move_command_index]);

                move_command_index += 1;

                continue;
            }
            (Some(Command::Script { lines }), CommandCode::SCRIPT_EXTRA) => {
                ensure!(event_command.parameters.len() == 1);
                let line = event_command.parameters[0]
                    .as_str()
                    .context("`line` is not a string")?
                    .to_string();

                lines.push(line);

                continue;
            }
            (_, CommandCode::NOP) => {
                Command::parse_nop(event_command).context("failed to parse NOP command")?
            }
            (_, CommandCode::SHOW_TEXT) => Command::parse_show_text(event_command)
                .context("failed to parse SHOW_TEXT command")?,
            (_, CommandCode::SHOW_CHOICES) => Command::parse_show_choices(event_command)
                .context("failed to parse SHOW_CHOICES command")?,
            (_, CommandCode::SHOW_SCROLLING_TEXT) => {
                let speed = event_command.parameters[0]
                    .as_i64()
                    .and_then(|value| u32::try_from(value).ok())
                    .context("`speed` is not a `u32`")?;
                let no_fast = event_command.parameters[1]
                    .as_bool()
                    .context("`no_fast` is not a `bool`")?;

                Command::ShowScrollingText {
                    speed,
                    no_fast,
                    lines: Vec::new(),
                }
            }
            (_, CommandCode::COMMENT) => {
                Command::parse_comment(event_command).context("failed to parse COMMENT command")?
            }
            (_, CommandCode::CONDITONAL_BRANCH) => Command::parse_conditional_branch(event_command)
                .context("failed to parse CONDITONAL_BRANCH command")?,
            (_, CommandCode::EXIT_EVENT_PROCESSING) => {
                ensure!(event_command.parameters.is_empty());
                Command::ExitEventProcessing
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
            (_, CommandCode::CONTROL_TIMER) => {
                ensure!(!event_command.parameters.is_empty());
                let is_start = event_command.parameters[0]
                    .as_i64()
                    .and_then(|value| u8::try_from(value).ok())
                    .context("`is_start` is not a `u8`")?;
                ensure!(is_start <= 1);
                let is_start = is_start == 0;

                let start_seconds = if is_start {
                    ensure!(event_command.parameters.len() == 2);
                    let start_seconds = event_command.parameters[1]
                        .as_i64()
                        .and_then(|value| u32::try_from(value).ok())
                        .context("`start_seconds` is not a `u32`")?;
                    Some(start_seconds)
                } else {
                    ensure!(event_command.parameters.len() == 1);
                    None
                };

                Command::ControlTimer { start_seconds }
            }
            (_, CommandCode::CHANGE_GOLD) => {
                ensure!(event_command.parameters.len() == 3);
                let is_add = event_command.parameters[0]
                    .as_i64()
                    .and_then(|value| u8::try_from(value).ok())
                    .context("`is_add` is not a `u8`")?;
                ensure!(is_add <= 1);
                let is_add = is_add == 0;
                let is_constant = event_command.parameters[1]
                    .as_i64()
                    .and_then(|value| u8::try_from(value).ok())
                    .context("`is_constant` is not a `u8`")?;
                ensure!(is_constant <= 1);
                let is_constant = is_constant == 0;
                let value = event_command.parameters[2]
                    .as_i64()
                    .and_then(|value| u32::try_from(value).ok())
                    .context("`value` is not a `u32`")?;
                let value = if is_constant {
                    MaybeRef::Constant(value)
                } else {
                    MaybeRef::Ref(value)
                };

                Command::ChangeGold { is_add, value }
            }
            (_, CommandCode::CHANGE_ITEMS) => Command::parse_change_items(event_command)
                .context("failed to parse CHANGE_ITEMS command")?,
            (_, CommandCode::CHANGE_ARMORS) => {
                ensure!(event_command.parameters.len() == 5);
                let armor_id = event_command.parameters[0]
                    .as_i64()
                    .and_then(|value| u32::try_from(value).ok())
                    .context("`armor_id` is not a `u32`")?;
                let is_add = event_command.parameters[1]
                    .as_i64()
                    .and_then(|value| u8::try_from(value).ok())
                    .context("`is_add` is not a `u8`")?;
                ensure!(is_add <= 1);
                let is_add = is_add == 0;
                let is_constant = event_command.parameters[2]
                    .as_i64()
                    .and_then(|value| u8::try_from(value).ok())
                    .context("`is_constant` is not a `u8`")?;
                ensure!(is_constant <= 1);
                let is_constant = is_constant == 0;
                let value = event_command.parameters[3]
                    .as_i64()
                    .and_then(|value| u32::try_from(value).ok())
                    .context("`value` is not a `u32`")?;
                let value = if is_constant {
                    MaybeRef::Constant(value)
                } else {
                    MaybeRef::Ref(value)
                };
                let include_equipped = event_command.parameters[4]
                    .as_bool()
                    .context("`include_equipped` is not a `bool`")?;

                Command::ChangeArmors {
                    armor_id,
                    is_add,
                    value,
                    include_equipped,
                }
            }
            (_, CommandCode::CHANGE_PARTY_MEMBER) => {
                ensure!(event_command.parameters.len() == 3);

                let actor_id = event_command.parameters[0]
                    .as_i64()
                    .and_then(|value| u32::try_from(value).ok())
                    .context("`actor_id` is not a `u32`")?;
                let is_add = event_command.parameters[1]
                    .as_i64()
                    .and_then(|value| u8::try_from(value).ok())
                    .context("`is_add` is not a `u8`")?;
                ensure!(is_add <= 1);
                let is_add = is_add == 0;
                let initialize = event_command.parameters[2]
                    .as_bool()
                    .context("`initialize` is not a `bool`")?;

                Command::ChangePartyMember {
                    actor_id,
                    is_add,
                    initialize,
                }
            }
            (_, CommandCode::CHANGE_SAVE_ACCESS) => {
                ensure!(event_command.parameters.len() == 1);
                let disable = event_command.parameters[0]
                    .as_i64()
                    .and_then(|value| u8::try_from(value).ok())
                    .context("`disable` is not a `u8`")?;
                ensure!(disable <= 1);
                let disable = disable == 0;

                Command::ChangeSaveAccess { disable }
            }
            (_, CommandCode::SET_EVENT_LOCATION) => {
                ensure!(event_command.parameters.len() == 5);

                let character_id = event_command.parameters[0]
                    .as_i64()
                    .and_then(|value| i32::try_from(value).ok())
                    .context("`value` is not an `i32`")?;
                let is_constant = event_command.parameters[1]
                    .as_i64()
                    .and_then(|value| u8::try_from(value).ok())
                    .context("`is_constant` is not a `u8`")?;
                ensure!(
                    is_constant <= 1,
                    "a non 0 or 1 `is_constant` value is currently unsupported"
                );
                let is_constant = is_constant == 0;
                let x = event_command.parameters[2]
                    .as_i64()
                    .and_then(|value| u32::try_from(value).ok())
                    .context("`x` is not a `u32`")?;
                let y = event_command.parameters[3]
                    .as_i64()
                    .and_then(|value| u32::try_from(value).ok())
                    .context("`x` is not a `u32`")?;
                let (x, y) = if is_constant {
                    (MaybeRef::Constant(x), MaybeRef::Constant(y))
                } else {
                    (MaybeRef::Ref(x), MaybeRef::Ref(y))
                };
                let direction = event_command.parameters[4]
                    .as_i64()
                    .and_then(|value| u8::try_from(value).ok())
                    .context("`direction` is not a `u8`")?;
                let direction = if direction == 0 {
                    None
                } else {
                    Some(direction)
                };

                Command::SetEventLocation {
                    character_id,
                    x,
                    y,
                    direction,
                }
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
            (_, CommandCode::CHANGE_TRANSPARENCY) => {
                ensure!(event_command.parameters.len() == 1);
                let value = event_command.parameters[0]
                    .as_i64()
                    .and_then(|value| u32::try_from(value).ok())
                    .context("`value` is not a `u32`")?;
                ensure!(value <= 1);

                let set_transparent = value == 0;
                Command::ChangeTransparency { set_transparent }
            }
            (_, CommandCode::CHANGE_PLAYER_FOLLOWERS) => {
                ensure!(event_command.parameters.len() == 1);

                let is_show = event_command.parameters[0]
                    .as_i64()
                    .and_then(|value| u8::try_from(value).ok())
                    .context("`is_show` is not a `u8`")?;
                ensure!(is_show <= 1);
                let is_show = is_show == 0;

                Command::ChangePlayerFollowers { is_show }
            }
            (_, CommandCode::FADEOUT_SCREEN) => Command::parse_fadeout_screen(event_command)
                .context("failed to parse FADEOUT_SCREEN command")?,
            (_, CommandCode::FADEIN_SCREEN) => Command::parse_fadein_screen(event_command)
                .context("failed to parse FADEIN_SCREEN command")?,
            (_, CommandCode::TINT_SCREEN) => {
                ensure!(event_command.parameters.len() == 3);
                let tone: [i16; 4] = serde_json::from_value(event_command.parameters[0].clone())
                    .context("invalid `tone` parameter")?;
                let duration = event_command.parameters[1]
                    .as_i64()
                    .and_then(|value| u32::try_from(value).ok())
                    .context("`duration` is not a `u32`")?;
                let wait = event_command.parameters[2]
                    .as_bool()
                    .context("`wait` is not a `bool`")?;

                Command::TintScreen {
                    tone,
                    duration,
                    wait,
                }
            }
            (_, CommandCode::FLASH_SCREEN) => {
                ensure!(event_command.parameters.len() == 3);
                let color: [u8; 4] = serde_json::from_value(event_command.parameters[0].clone())
                    .context("invalid `color` parameter")?;
                let duration = event_command.parameters[1]
                    .as_i64()
                    .and_then(|value| u32::try_from(value).ok())
                    .context("`duration` is not a `u32`")?;
                let wait = event_command.parameters[2]
                    .as_bool()
                    .context("`wait` is not a `bool`")?;

                Command::FlashScreen {
                    color,
                    duration,
                    wait,
                }
            }
            (_, CommandCode::SHAKE_SCREEN) => Command::parse_shake_screen(event_command)
                .context("failed to parse SHAKE_SCREEN command")?,
            (_, CommandCode::WAIT) => {
                Command::parse_wait(event_command).context("failed to parse WAIT command")?
            }
            (_, CommandCode::SHOW_PICTURE) => Command::parse_show_picture(event_command)
                .context("failed to parse SHOW_PICTURE command")?,
            (_, CommandCode::ERASE_PICTURE) => Command::parse_erase_picture(event_command)
                .context("failed to parse ERASE_PICTURE command")?,
            (_, CommandCode::PLAY_BGM) => Command::parse_play_bgm(event_command)
                .context("failed to parse PLAY_BGM command")?,
            (_, CommandCode::FADEOUT_BGM) => Command::parse_fadeout_bgm(event_command)
                .context("failed to parse FADEOUT_BGM command")?,
            (_, CommandCode::SAVE_BGM) => {
                ensure!(event_command.parameters.is_empty());
                Command::SaveBgm
            }
            (_, CommandCode::PLAY_BGS) => {
                ensure!(event_command.parameters.len() == 1);
                let audio: rpgmv_types::AudioFile =
                    serde_json::from_value(event_command.parameters[0].clone())
                        .context("invalid `audio` parameter")?;

                Command::PlayBgs { audio }
            }
            (_, CommandCode::FADEOUT_BGS) => {
                ensure!(event_command.parameters.len() == 1);

                let duration = event_command.parameters[0]
                    .as_i64()
                    .and_then(|value| u32::try_from(value).ok())
                    .context("`duration` is not a `u32`")?;

                Command::FadeoutBgs { duration }
            }
            (_, CommandCode::RESUME_BGM) => {
                ensure!(event_command.parameters.is_empty());
                Command::ResumeBgm
            }
            (_, CommandCode::PLAY_SE) => {
                Command::parse_play_se(event_command).context("failed to parse PLAY_SE command")?
            }
            (_, CommandCode::GET_LOCATION_INFO) => {
                ensure!(event_command.parameters.len() == 5);

                let variable_id = event_command.parameters[0]
                    .as_i64()
                    .and_then(|value| u32::try_from(value).ok())
                    .context("`variable_id` is not a `u32`")?;

                let kind = event_command.parameters[1]
                    .as_i64()
                    .and_then(|value| u8::try_from(value).ok())
                    .context("`kind` is not a `u8`")?;
                let kind = GetLocationInfoKind::from_u8(kind)?;

                let is_constant = event_command.parameters[2]
                    .as_i64()
                    .and_then(|value| u8::try_from(value).ok())
                    .context("`is_constant` is not a `u8`")?;
                ensure!(is_constant <= 1);
                let is_constant = is_constant == 0;

                let x = event_command.parameters[3]
                    .as_i64()
                    .and_then(|value| u32::try_from(value).ok())
                    .context("`x` is not a `u32`")?;
                let y = event_command.parameters[4]
                    .as_i64()
                    .and_then(|value| u32::try_from(value).ok())
                    .context("`x` is not a `u32`")?;
                let (x, y) = if is_constant {
                    (MaybeRef::Constant(x), MaybeRef::Constant(y))
                } else {
                    (MaybeRef::Ref(x), MaybeRef::Ref(y))
                };

                Command::GetLocationInfo {
                    variable_id,
                    kind,
                    x,
                    y,
                }
            }
            (_, CommandCode::BATTLE_PROCESSING) => {
                ensure!(event_command.parameters.len() == 4);
                let is_constant = event_command.parameters[0]
                    .as_i64()
                    .and_then(|value| u8::try_from(value).ok())
                    .context("`is_constant` is not a `u8`")?;
                ensure!(is_constant <= 1);
                // TODO: This can be another value, meaning the troop id is random.
                let is_constant = is_constant == 0;
                let troop_id = event_command.parameters[1]
                    .as_i64()
                    .and_then(|value| u32::try_from(value).ok())
                    .context("`troop_id` is not a `u32`")?;
                let troop_id = if is_constant {
                    MaybeRef::Constant(troop_id)
                } else {
                    MaybeRef::Ref(troop_id)
                };
                let can_escape = event_command.parameters[2]
                    .as_bool()
                    .context("`can_escape` is not a `bool`")?;
                let can_lose = event_command.parameters[3]
                    .as_bool()
                    .context("`can_lose` is not a `bool`")?;

                Command::BattleProcessing {
                    troop_id,
                    can_escape,
                    can_lose,
                }
            }
            (_, CommandCode::NAME_INPUT_PROCESSING) => {
                Command::parse_name_input_processing(event_command)
                    .context("failed to parse NAME_INPUT_PROCESSING command")?
            }
            (_, CommandCode::CHANGE_HP) => {
                ensure!(event_command.parameters.len() == 6);

                let is_actor_constant = event_command.parameters[0]
                    .as_i64()
                    .and_then(|value| u8::try_from(value).ok())
                    .context("`is_actor_constant` is not a `u8`")?;
                ensure!(is_actor_constant <= 1);
                let is_actor_constant = is_actor_constant == 0;
                let actor_id = event_command.parameters[1]
                    .as_i64()
                    .and_then(|value| u32::try_from(value).ok())
                    .context("`actor_id` is not a `u32`")?;
                let actor_id = if is_actor_constant {
                    MaybeRef::Constant(actor_id)
                } else {
                    MaybeRef::Ref(actor_id)
                };
                let is_add = event_command.parameters[2]
                    .as_i64()
                    .and_then(|value| u8::try_from(value).ok())
                    .context("`is_add` is not a `u8`")?;
                ensure!(is_add <= 1);
                let is_add = is_add == 0;
                let is_constant = event_command.parameters[3]
                    .as_i64()
                    .and_then(|value| u8::try_from(value).ok())
                    .context("`is_constant` is not a `u8`")?;
                ensure!(is_constant <= 1);
                let is_constant = is_constant == 0;
                let value = event_command.parameters[4]
                    .as_i64()
                    .and_then(|value| u32::try_from(value).ok())
                    .context("`value` is not a `u32`")?;
                let value = if is_constant {
                    MaybeRef::Constant(value)
                } else {
                    MaybeRef::Ref(value)
                };
                let allow_death = event_command.parameters[5]
                    .as_bool()
                    .context("`allow_death` is not a `u32`")?;

                Command::ChangeHp {
                    actor_id,
                    is_add,
                    value,
                    allow_death,
                }
            }
            (_, CommandCode::CHANGE_MP) => Command::parse_change_mp(event_command)
                .context("failed to parse CHANGE_MP command")?,
            (_, CommandCode::CHANGE_STATE) => {
                ensure!(event_command.parameters.len() == 4);
                let is_constant = event_command.parameters[0]
                    .as_i64()
                    .and_then(|value| u8::try_from(value).ok())
                    .context("`is_constant` is not a `u8`")?;
                ensure!(is_constant <= 1);
                let is_constant = is_constant == 0;
                let actor_id = event_command.parameters[1]
                    .as_i64()
                    .and_then(|value| u32::try_from(value).ok())
                    .context("`actor_id` is not a `u32`")?;
                let actor_id = if is_constant {
                    MaybeRef::Constant(actor_id)
                } else {
                    MaybeRef::Ref(actor_id)
                };
                let is_add_state = event_command.parameters[2]
                    .as_i64()
                    .and_then(|value| u8::try_from(value).ok())
                    .context("`is_add_state` is not a `u8`")?;
                ensure!(is_add_state <= 1);
                let is_add_state = is_add_state == 0;
                let state_id = event_command.parameters[3]
                    .as_i64()
                    .and_then(|value| u32::try_from(value).ok())
                    .context("`state_id` is not a `u32`")?;

                Command::ChangeState {
                    actor_id,
                    is_add_state,
                    state_id,
                }
            }
            (_, CommandCode::CHANGE_LEVEL) => {
                ensure!(event_command.parameters.len() == 6);

                let is_actor_constant = event_command.parameters[0]
                    .as_i64()
                    .and_then(|value| u8::try_from(value).ok())
                    .context("`is_actor_constant` is not a `u8`")?;
                ensure!(is_actor_constant <= 1);
                let is_actor_constant = is_actor_constant == 0;
                let actor_id = event_command.parameters[1]
                    .as_i64()
                    .and_then(|value| u32::try_from(value).ok())
                    .context("`actor_id` is not a `u32`")?;
                let actor_id = if is_actor_constant {
                    MaybeRef::Constant(actor_id)
                } else {
                    MaybeRef::Ref(actor_id)
                };
                let is_add = event_command.parameters[2]
                    .as_i64()
                    .and_then(|value| u8::try_from(value).ok())
                    .context("`is_add` is not a `u8`")?;
                ensure!(is_add <= 1);
                let is_add = is_add == 0;
                let is_constant = event_command.parameters[3]
                    .as_i64()
                    .and_then(|value| u8::try_from(value).ok())
                    .context("`is_constant` is not a `u8`")?;
                ensure!(is_constant <= 1);
                let is_constant = is_constant == 0;
                let value = event_command.parameters[4]
                    .as_i64()
                    .and_then(|value| u32::try_from(value).ok())
                    .context("`value` is not a `u32`")?;
                let value = if is_constant {
                    MaybeRef::Constant(value)
                } else {
                    MaybeRef::Ref(value)
                };
                let show_level_up = event_command.parameters[5]
                    .as_bool()
                    .context("`show_level_up` is not a `bool`")?;

                Command::ChangeLevel {
                    actor_id,
                    is_add,
                    value,
                    show_level_up,
                }
            }
            (_, CommandCode::CHANGE_SKILL) => {
                ensure!(event_command.parameters.len() == 4);
                let is_constant = event_command.parameters[0]
                    .as_i64()
                    .and_then(|value| u8::try_from(value).ok())
                    .context("`is_constant` is not a `u8`")?;
                ensure!(is_constant <= 1);
                let is_constant = is_constant == 0;
                let actor_id = event_command.parameters[1]
                    .as_i64()
                    .and_then(|value| u32::try_from(value).ok())
                    .context("`actor_id` is not a `u32`")?;
                let actor_id = if is_constant {
                    MaybeRef::Constant(actor_id)
                } else {
                    MaybeRef::Ref(actor_id)
                };
                let is_learn_skill = event_command.parameters[2]
                    .as_i64()
                    .and_then(|value| u8::try_from(value).ok())
                    .context("`is_learn_skill` is not a `u8`")?;
                ensure!(is_learn_skill <= 1);
                let is_learn_skill = is_learn_skill == 0;
                let skill_id = event_command.parameters[3]
                    .as_i64()
                    .and_then(|value| u32::try_from(value).ok())
                    .context("`skill_id` is not a `u32`")?;

                Command::ChangeSkill {
                    actor_id,
                    is_learn_skill,
                    skill_id,
                }
            }
            (_, CommandCode::CHANGE_CLASS) => {
                ensure!(event_command.parameters.len() == 3);

                let actor_id = event_command.parameters[0]
                    .as_i64()
                    .and_then(|value| u32::try_from(value).ok())
                    .context("`actor_id` is not a `u32`")?;
                let class_id = event_command.parameters[1]
                    .as_i64()
                    .and_then(|value| u32::try_from(value).ok())
                    .context("`class_id` is not a `u32`")?;
                let keep_exp = event_command.parameters[2]
                    .as_bool()
                    .context("`keep_exp` is not a `bool`")?;

                Command::ChangeClass {
                    actor_id,
                    class_id,
                    keep_exp,
                }
            }
            (_, CommandCode::CHANGE_ACTOR_IMAGES) => {
                ensure!(event_command.parameters.len() == 6);

                let actor_id = event_command.parameters[0]
                    .as_i64()
                    .and_then(|value| u32::try_from(value).ok())
                    .context("`actor_id` is not a `u32`")?;
                let character_name = event_command.parameters[1]
                    .as_str()
                    .context("`character_name` is not a str")?
                    .to_string();
                let character_index = event_command.parameters[2]
                    .as_i64()
                    .and_then(|value| u32::try_from(value).ok())
                    .context("`character_index` is not a `u32`")?;
                let face_name = event_command.parameters[3]
                    .as_str()
                    .context("`face_name` is not a str")?
                    .to_string();
                let face_index = event_command.parameters[4]
                    .as_i64()
                    .and_then(|value| u32::try_from(value).ok())
                    .context("`face_index` is not a `u32`")?;
                let battler_name = event_command.parameters[5]
                    .as_str()
                    .context("`battler_name` is not a str")?
                    .to_string();

                Command::ChangeActorImages {
                    actor_id,
                    character_name,
                    character_index,
                    face_name,
                    face_index,
                    battler_name,
                }
            }
            (_, CommandCode::FORCE_ACTION) => {
                ensure!(event_command.parameters.len() == 4);
                let is_enemy = event_command.parameters[0]
                    .as_i64()
                    .and_then(|value| u8::try_from(value).ok())
                    .context("`enemy` is not a `u8`")?;
                ensure!(is_enemy <= 1);
                let is_enemy = is_enemy == 0;
                let id = event_command.parameters[1]
                    .as_i64()
                    .and_then(|value| u32::try_from(value).ok())
                    .context("`id` is not a `u32`")?;
                let skill_id = event_command.parameters[1]
                    .as_i64()
                    .and_then(|value| u32::try_from(value).ok())
                    .context("`skill_id` is not a `u32`")?;
                let target_index = event_command.parameters[1]
                    .as_i64()
                    .and_then(|value| u32::try_from(value).ok())
                    .context("`target_index` is not a `u32`")?;

                Command::ForceAction {
                    is_enemy,
                    id,
                    skill_id,
                    target_index,
                }
            }
            (_, CommandCode::ABORT_BATTLE) => {
                ensure!(event_command.parameters.is_empty());

                Command::AbortBattle
            }
            (_, CommandCode::RETURN_TO_TITLE_SCREEN) => {
                ensure!(event_command.parameters.is_empty());

                Command::ReturnToTitleScreen
            }
            (_, CommandCode::SCRIPT) => {
                ensure!(event_command.parameters.len() == 1);
                let line = event_command.parameters[0]
                    .as_str()
                    .context("`line` is not a string")?
                    .to_string();

                Command::Script { lines: vec![line] }
            }
            (_, CommandCode::PLUGIN_COMMAND) => {
                ensure!(event_command.parameters.len() == 1);
                let params = event_command.parameters[0]
                    .as_str()
                    .context("`params` is not a string")?;

                Command::PluginCommand {
                    params: params.split(' ').map(|value| value.to_string()).collect(),
                }
            }
            (_, CommandCode::WHEN) => {
                Command::parse_when(event_command).context("failed to parse WHEN command")?
            }
            (_, CommandCode::WHEN_CANCEL) => {
                ensure!(event_command.parameters.len() == 2);
                let choice_index = event_command.parameters[0]
                    .as_i64()
                    .and_then(|value| u32::try_from(value).ok())
                    .context("`choice_index` is not a `u32`")?;
                let choice_name: Option<String> =
                    serde_json::from_value(event_command.parameters[1].clone())
                        .context("`choice_name` is not a string")?;

                Command::WhenCancel {
                    choice_index,
                    choice_name,
                }
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
            (_, CommandCode::IF_WIN) => {
                ensure!(event_command.parameters.is_empty());
                Command::IfWin
            }
            (_, CommandCode::IF_ESCAPE) => {
                ensure!(event_command.parameters.is_empty());
                Command::IfEscape
            }
            (_, CommandCode::IF_LOSE) => {
                ensure!(event_command.parameters.is_empty());
                Command::IfLose
            }
            (_, CommandCode::BATTLE_RESULT_END) => {
                ensure!(event_command.parameters.is_empty());
                Command::BattleResultEnd
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
