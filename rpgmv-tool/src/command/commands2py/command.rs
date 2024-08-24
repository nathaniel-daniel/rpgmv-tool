mod code;

use self::code::CommandCode;
use anyhow::bail;
use anyhow::ensure;
use anyhow::Context;

#[derive(Debug, Copy, Clone)]
enum ConditionalBranchKind {
    Switch = 0,
    Variable = 1,
    SelfSwitch = 2,
    Timer = 3,
    Actor = 4,
    Enemy = 5,
    Character = 6,
    Gold = 7,
    Item = 8,
    Weapon = 9,
    Armor = 10,
    Button = 11,
    Script = 12,
}

impl ConditionalBranchKind {
    /// Get this from a u8.
    pub fn from_u8(value: u8) -> anyhow::Result<Self> {
        match value {
            0 => Ok(Self::Switch),
            1 => Ok(Self::Variable),
            2 => Ok(Self::SelfSwitch),
            3 => Ok(Self::Timer),
            4 => Ok(Self::Actor),
            5 => Ok(Self::Enemy),
            6 => Ok(Self::Character),
            7 => Ok(Self::Gold),
            8 => Ok(Self::Item),
            9 => Ok(Self::Weapon),
            10 => Ok(Self::Armor),
            11 => Ok(Self::Button),
            12 => Ok(Self::Script),
            _ => bail!("{value} is not a valid ConditionalBranchKind"),
        }
    }

    /*
    /// Get this as a u8.
    pub fn as_u8(self) -> u8 {
        self as u8
    }
    */
}

/// The type of actor check
#[derive(Debug, Copy, Clone)]
pub enum ConditionalBranchKindActorCheck {
    InParty = 0,
    Name = 1,
    Class = 2,
    Skill = 3,
    Weapon = 4,
    Armor = 5,
    State = 6,
}

impl ConditionalBranchKindActorCheck {
    /// Get this from a u8.
    pub fn from_u8(value: u8) -> anyhow::Result<Self> {
        match value {
            0 => Ok(Self::InParty),
            1 => Ok(Self::Name),
            2 => Ok(Self::Class),
            3 => Ok(Self::Skill),
            4 => Ok(Self::Weapon),
            5 => Ok(Self::Armor),
            6 => Ok(Self::State),
            _ => bail!("{value} is not a valid ConditionalBranchKindActorCheck"),
        }
    }
}

/// The type of enemy check
#[derive(Debug, Copy, Clone)]
enum ConditionalBranchKindEnemyCheck {
    Appeared = 0,
    State = 1,
}

impl ConditionalBranchKindEnemyCheck {
    /// Get this from a u8.
    pub fn from_u8(value: u8) -> anyhow::Result<Self> {
        match value {
            0 => Ok(Self::Appeared),
            1 => Ok(Self::State),
            _ => bail!("{value} is not a valid ConditionalBranchKindEnemyCheck"),
        }
    }
}

/// The type of gold check
#[derive(Debug, Copy, Clone)]
pub enum ConditionalBranchKindGoldCheck {
    Gte = 0,
    Lte = 1,
    Lt = 2,
}

impl ConditionalBranchKindGoldCheck {
    /// Get this from a u8.
    pub fn from_u8(value: u8) -> anyhow::Result<Self> {
        match value {
            0 => Ok(Self::Gte),
            1 => Ok(Self::Lte),
            2 => Ok(Self::Lt),
            _ => bail!("{value} is not a valid ConditionalBranchKindGoldCheck"),
        }
    }

    /// Get this as a string.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Gte => ">=",
            Self::Lte => "<=",
            Self::Lt => "<",
        }
    }
}

/// The type of variable compare operation
#[derive(Debug, Copy, Clone)]
pub enum ConditionalBranchVariableOperation {
    /// ==
    EqualTo = 0,

    /// >=
    Gte = 1,

    /// <=
    Lte = 2,

    /// >
    Gt = 3,

    /// <
    Lt = 4,

    /// !=
    Neq = 5,
}

impl ConditionalBranchVariableOperation {
    /// Get this from a u8.
    pub fn from_u8(value: u8) -> anyhow::Result<Self> {
        match value {
            0 => Ok(Self::EqualTo),
            1 => Ok(Self::Gte),
            2 => Ok(Self::Lte),
            3 => Ok(Self::Gt),
            4 => Ok(Self::Lt),
            5 => Ok(Self::Neq),
            _ => bail!("{value} is not a valid ConditionalBranchVariableOperation"),
        }
    }
    /*
    /// Get this as a u8.
    pub fn as_u8(self) -> u8 {
        self as u8
    }
    */

    /// Get this as a str.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::EqualTo => "==",
            Self::Gte => ">=",
            Self::Lte => "<=",
            Self::Gt => ">",
            Self::Lt => "<",
            Self::Neq => "!=",
        }
    }
}

#[derive(Debug, Copy, Clone)]
enum ControlVariablesOperation {
    Const = 0,
    Var = 1,
    Random = 2,
    GameData = 3,
    Script = 4,
}

impl ControlVariablesOperation {
    /// Get this from a u8.
    pub fn from_u8(value: u8) -> anyhow::Result<Self> {
        match value {
            0 => Ok(Self::Const),
            1 => Ok(Self::Var),
            2 => Ok(Self::Random),
            3 => Ok(Self::GameData),
            4 => Ok(Self::Script),
            _ => bail!("{value} is not a valid ControlVariablesOperation"),
        }
    }
}

/// The type of variable operation.
#[derive(Debug, Copy, Clone)]
pub enum OperateVariableOperation {
    /// =
    Set = 0,

    /// +=
    Add = 1,

    /// -=
    Sub = 2,

    /// *=
    Mul = 3,

    /// /=
    Div = 4,
}

impl OperateVariableOperation {
    /// Get this from a u8.
    pub fn from_u8(value: u8) -> anyhow::Result<Self> {
        match value {
            0 => Ok(Self::Set),
            1 => Ok(Self::Add),
            2 => Ok(Self::Sub),
            3 => Ok(Self::Mul),
            4 => Ok(Self::Div),
            _ => bail!("{value} is not a valid OperateVariableOperation"),
        }
    }

    /// Get this as a str.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Set => "=",
            Self::Add => "+=",
            Self::Sub => "-=",
            Self::Mul => "*=",
            Self::Div => "/=",
        }
    }
}

#[derive(Debug, Copy, Clone)]
enum GameDataOperandKind {
    Item = 0,
    Weapon = 1,
    Armor = 2,
    Actor = 3,
    Enemy = 4,
    Character = 5,
    Party = 6,
    Other = 7,
}

impl GameDataOperandKind {
    /// Get this from a u8.
    pub fn from_u8(value: u8) -> anyhow::Result<Self> {
        match value {
            0 => Ok(Self::Item),
            1 => Ok(Self::Weapon),
            2 => Ok(Self::Armor),
            3 => Ok(Self::Actor),
            4 => Ok(Self::Enemy),
            5 => Ok(Self::Character),
            6 => Ok(Self::Party),
            7 => Ok(Self::Other),
            _ => bail!("{value} is not a valid GameDataOperandKind"),
        }
    }
}

#[derive(Debug, Copy, Clone)]
enum GameDataOperandKindOtherCheck {
    MapId = 0,
    PartyMembers = 1,
    Gold = 2,
    Steps = 3,
    PlayTime = 4,
    Timer = 5,
    SaveCount = 6,
    BattleCount = 7,
    WinCount = 8,
    EscapeCount = 9,
}

impl GameDataOperandKindOtherCheck {
    /// Get this from a u8.
    pub fn from_u8(value: u8) -> anyhow::Result<Self> {
        match value {
            0 => Ok(Self::MapId),
            1 => Ok(Self::PartyMembers),
            2 => Ok(Self::Gold),
            3 => Ok(Self::Steps),
            4 => Ok(Self::PlayTime),
            5 => Ok(Self::Timer),
            6 => Ok(Self::SaveCount),
            7 => Ok(Self::BattleCount),
            8 => Ok(Self::WinCount),
            9 => Ok(Self::EscapeCount),
            _ => bail!("{value} is not a valid GameDataOperandKindOtherCheck"),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum GameDataOperandKindActorCheck {
    Level = 0,
    Exp = 1,
}

impl GameDataOperandKindActorCheck {
    /// Get this from a u8.
    pub fn from_u8(value: u8) -> anyhow::Result<Self> {
        match value {
            0 => Ok(Self::Level),
            1 => Ok(Self::Exp),
            _ => bail!("{value} is not a valid GameDataOperandKindActorCheck"),
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
    ConditionalBranch(ConditionalBranchCommand),
    ExitEventProcessing,
    CommonEvent {
        id: u32,
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
    ChangePartyMember {
        actor_id: u32,
        is_add: bool,
        initialize: bool,
    },
    ChangeSaveAccess {
        disable: bool,
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
    PlaySe {
        audio: rpgmv_types::AudioFile,
    },
    BattleProcessing {
        troop_id: MaybeRef<u32>,
        can_escape: bool,
        can_lose: bool,
    },
    ChangeSkill {
        actor_id: MaybeRef<u32>,
        is_learn_skill: bool,
        skill_id: u32,
    },
    ChangeState {
        actor_id: MaybeRef<u32>,
        is_add_state: bool,
        state_id: u32,
    },
    ChangeActorImages {
        actor_id: u32,
        character_name: String,
        character_index: u32,
        face_name: String,
        face_index: u32,
        battler_name: String,
    },
    Script {
        lines: Vec<String>,
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
    fn parse_show_text(event_command: &rpgmv_types::EventCommand) -> anyhow::Result<Self> {
        ensure!(event_command.parameters.len() == 4);

        let face_name = event_command.parameters[0]
            .as_str()
            .context("`face_name` is not a string")?
            .to_string();
        let face_index = event_command.parameters[1]
            .as_i64()
            .and_then(|n| u32::try_from(n).ok())
            .context("`face_index` is not a `u32`")?;
        let background = event_command.parameters[2]
            .as_i64()
            .and_then(|n| u32::try_from(n).ok())
            .context("`background` is not a string")?;
        let position_type = event_command.parameters[3]
            .as_i64()
            .and_then(|n| u32::try_from(n).ok())
            .context("`position_type` is not a string")?;

        Ok(Command::ShowText {
            face_name,
            face_index,
            background,
            position_type,
            lines: Vec::new(),
        })
    }

    fn parse_conditional_branch(event_command: &rpgmv_types::EventCommand) -> anyhow::Result<Self> {
        ensure!(!event_command.parameters.is_empty());
        let kind = event_command.parameters[0]
            .as_i64()
            .and_then(|value| u8::try_from(value).ok())
            .context("`kind` is not a `u32`")?;
        let kind = ConditionalBranchKind::from_u8(kind)?;

        let inner = match kind {
            ConditionalBranchKind::Switch => {
                ensure!(event_command.parameters.len() == 3);

                let id = event_command.parameters[1]
                    .as_i64()
                    .and_then(|value| u32::try_from(value).ok())
                    .context("`id` is not a `u32`")?;
                let check_true = event_command.parameters[2]
                    .as_i64()
                    .and_then(|value| u8::try_from(value).ok())
                    .context("`check_true` is not a `u32`")?;
                ensure!(check_true <= 1);
                let check_true = check_true == 0;

                ConditionalBranchCommand::Switch { id, check_true }
            }
            ConditionalBranchKind::Variable => {
                ensure!(event_command.parameters.len() == 5);

                let lhs_id = event_command.parameters[1]
                    .as_i64()
                    .and_then(|value| u32::try_from(value).ok())
                    .context("`lhs_id` is not a `u32`")?;
                let is_constant = event_command.parameters[2]
                    .as_i64()
                    .and_then(|value| u8::try_from(value).ok())
                    .context("`is_constant` is not a `u32`")?;
                let is_constant = is_constant == 0;
                let rhs_id = event_command.parameters[3]
                    .as_i64()
                    .and_then(|value| u32::try_from(value).ok())
                    .context("`rhs_id` is not a `u32`")?;
                let rhs_id = if is_constant {
                    MaybeRef::Constant(rhs_id)
                } else {
                    MaybeRef::Ref(rhs_id)
                };
                let operation = event_command.parameters[4]
                    .as_i64()
                    .and_then(|value| u8::try_from(value).ok())
                    .context("`operation` is not a `u8`")?;
                let operation = ConditionalBranchVariableOperation::from_u8(operation)?;

                ConditionalBranchCommand::Variable {
                    lhs_id,
                    rhs_id,
                    operation,
                }
            }
            ConditionalBranchKind::Actor => {
                ensure!(event_command.parameters.len() >= 3);
                let actor_id = event_command.parameters[1]
                    .as_i64()
                    .and_then(|value| u32::try_from(value).ok())
                    .context("`actor_id` is not a `u32`")?;
                let check = event_command.parameters[2]
                    .as_i64()
                    .and_then(|value| u8::try_from(value).ok())
                    .context("`check` is not a `u8`")?;
                let check = ConditionalBranchKindActorCheck::from_u8(check)?;

                match check {
                    ConditionalBranchKindActorCheck::InParty => {
                        ensure!(event_command.parameters.len() == 3);
                        ConditionalBranchCommand::ActorInParty { actor_id }
                    }
                    ConditionalBranchKindActorCheck::Armor => {
                        ensure!(event_command.parameters.len() == 4);

                        let armor_id = event_command.parameters[3]
                            .as_i64()
                            .and_then(|value| u32::try_from(value).ok())
                            .context("`armor_id` is not a `u32`")?;

                        ConditionalBranchCommand::ActorArmor { actor_id, armor_id }
                    }
                    ConditionalBranchKindActorCheck::State => {
                        ensure!(event_command.parameters.len() == 4);
                        let state_id = event_command.parameters[3]
                            .as_i64()
                            .and_then(|value| u32::try_from(value).ok())
                            .context("`state_id` is not a `u32`")?;
                        ConditionalBranchCommand::ActorState { actor_id, state_id }
                    }
                    _ => {
                        bail!("ConditionalBranchKindActorCheck {check:?} is not supported")
                    }
                }
            }
            ConditionalBranchKind::Enemy => {
                ensure!(event_command.parameters.len() >= 3);
                let enemy_index = event_command.parameters[1]
                    .as_i64()
                    .and_then(|value| u32::try_from(value).ok())
                    .context("`enemy_index` is not a `u32`")?;
                let check = event_command.parameters[2]
                    .as_i64()
                    .and_then(|value| u8::try_from(value).ok())
                    .context("`check` is not a `u8`")?;
                let check = ConditionalBranchKindEnemyCheck::from_u8(check)?;

                match check {
                    ConditionalBranchKindEnemyCheck::State => {
                        ensure!(event_command.parameters.len() == 4);

                        let state_id = event_command.parameters[2]
                            .as_i64()
                            .and_then(|value| u32::try_from(value).ok())
                            .context("`check` is not a `u32`")?;

                        ConditionalBranchCommand::EnemyState {
                            enemy_index,
                            state_id,
                        }
                    }
                    _ => {
                        bail!("ConditionalBranchKindEnemyCheck {check:?} is not supported")
                    }
                }
            }
            ConditionalBranchKind::Character => {
                ensure!(event_command.parameters.len() == 3);
                let character_id = event_command.parameters[1]
                    .as_i64()
                    .and_then(|value| i32::try_from(value).ok())
                    .context("`character_id` is not an `i32`")?;
                let direction = event_command.parameters[2]
                    .as_i64()
                    .and_then(|value| u8::try_from(value).ok())
                    .context("`direction` is not a `u8`")?;

                ConditionalBranchCommand::Character {
                    character_id,
                    direction,
                }
            }
            ConditionalBranchKind::Gold => {
                ensure!(event_command.parameters.len() == 3);
                let value = event_command.parameters[1]
                    .as_i64()
                    .and_then(|value| u32::try_from(value).ok())
                    .context("`value` is not a `u32`")?;
                let check = event_command.parameters[2]
                    .as_i64()
                    .and_then(|value| u8::try_from(value).ok())
                    .context("`check` is not a `u8`")?;
                let check = ConditionalBranchKindGoldCheck::from_u8(check)?;

                ConditionalBranchCommand::Gold { value, check }
            }
            ConditionalBranchKind::Item => {
                ensure!(event_command.parameters.len() == 2);
                let item_id = event_command.parameters[1]
                    .as_i64()
                    .and_then(|value| u32::try_from(value).ok())
                    .context("`item_id` is not a `u32`")?;

                ConditionalBranchCommand::Item { item_id }
            }
            ConditionalBranchKind::Script => {
                ensure!(event_command.parameters.len() == 2);
                let value = event_command.parameters[1]
                    .as_str()
                    .context("`value` is not a string")?
                    .to_string();

                ConditionalBranchCommand::Script { value }
            }
            _ => bail!("ConditionalBranchKind {kind:?} is not supported"),
        };

        Ok(Command::ConditionalBranch(inner))
    }

    fn parse_transfer_player(event_command: &rpgmv_types::EventCommand) -> anyhow::Result<Self> {
        ensure!(event_command.parameters.len() == 6);
        let is_constant = event_command.parameters[0]
            .as_i64()
            .and_then(|value| u8::try_from(value).ok())
            .context("`is_constant` is not a `u8`")?;
        ensure!(is_constant <= 1);
        let is_constant = is_constant == 0;
        let map_id = event_command.parameters[1]
            .as_i64()
            .and_then(|value| u32::try_from(value).ok())
            .context("`y` is not a `u32`")?;
        let x = event_command.parameters[2]
            .as_i64()
            .and_then(|value| u32::try_from(value).ok())
            .context("`x` is not a `u32`")?;
        let y = event_command.parameters[3]
            .as_i64()
            .and_then(|value| u32::try_from(value).ok())
            .context("`y` is not a `u32`")?;
        let direction = event_command.parameters[3]
            .as_i64()
            .and_then(|value| u8::try_from(value).ok())
            .context("`direction` is not a `u8`")?;
        let fade_type = event_command.parameters[3]
            .as_i64()
            .and_then(|value| u8::try_from(value).ok())
            .context("`fade_type` is not a `u8`")?;

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

        Ok(Command::ShowPicture {
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
}

#[derive(Debug)]
pub enum ConditionalBranchCommand {
    Switch {
        id: u32,
        check_true: bool,
    },
    Variable {
        lhs_id: u32,
        rhs_id: MaybeRef<u32>,
        operation: ConditionalBranchVariableOperation,
    },
    ActorInParty {
        actor_id: u32,
    },
    ActorArmor {
        actor_id: u32,
        armor_id: u32,
    },
    ActorState {
        actor_id: u32,
        state_id: u32,
    },
    EnemyState {
        enemy_index: u32,
        state_id: u32,
    },
    Character {
        character_id: i32,
        direction: u8,
    },
    Gold {
        value: u32,
        check: ConditionalBranchKindGoldCheck,
    },
    Item {
        item_id: u32,
    },
    Script {
        value: String,
    },
}

#[derive(Debug)]
pub enum ControlVariablesValue {
    Constant { value: u32 },
    Variable { id: u32 },
    Random { start: u32, stop: u32 },
    GameData(ControlVariablesValueGameData),
}

#[derive(Debug)]
pub enum ControlVariablesValueGameData {
    NumItems { item_id: u32 },
    ActorLevel { actor_id: u32 },
    MapId,
    Gold,
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
                ensure!(event_command.parameters.len() == 1);
                let line = event_command.parameters[0]
                    .as_str()
                    .context("`line` is not a string")?
                    .to_string();

                lines.push(line);

                continue;
            }
            (
                Some(Command::ShowScrollingText { lines, .. }),
                CommandCode::SHOW_SCROLLING_TEXT_EXTRA,
            ) => {
                ensure!(event_command.parameters.len() == 1);
                let line = event_command.parameters[0]
                    .as_str()
                    .context("`line` is not a string")?
                    .to_string();

                lines.push(line);

                continue;
            }
            (
                Some(Command::SetMovementRoute { route, .. }),
                CommandCode::SET_MOVEMENT_ROUTE_EXTRA,
            ) if move_command_index < route.list.len() => {
                ensure!(event_command.parameters.len() == 1);
                let command: rpgmv_types::MoveCommand =
                    serde_json::from_value(event_command.parameters[0].clone())
                        .context("invalid `command` parameter")?;

                ensure!(command == route.list[move_command_index]);

                move_command_index += 1;

                continue;
            }
            (_, CommandCode::NOP) => {
                ensure!(event_command.parameters.is_empty());

                Command::Nop
            }
            (_, CommandCode::SHOW_TEXT) => Command::parse_show_text(event_command)
                .context("failed to parse SHOW_TEXT command")?,
            (_, CommandCode::SHOW_CHOICES) => {
                ensure!(event_command.parameters.len() == 5);

                let choices: Vec<String> =
                    serde_json::from_value(event_command.parameters[0].clone())
                        .context("invalid `choices` parameter")?;
                let cancel_type = event_command.parameters[1]
                    .as_i64()
                    .and_then(|value| i32::try_from(value).ok())
                    .context("`cancel_type` is not an `i32`")?;
                let default_type = event_command.parameters[2]
                    .as_i64()
                    .context("`default_type` is not an `i64`")?;
                let position_type = event_command.parameters[3]
                    .as_i64()
                    .and_then(|value| u32::try_from(value).ok())
                    .context("`position_type` is not a `u32`")?;
                let background = event_command.parameters[4]
                    .as_i64()
                    .and_then(|value| u32::try_from(value).ok())
                    .context("`background` is not a `u32`")?;

                Command::ShowChoices {
                    choices,
                    cancel_type,
                    default_type,
                    position_type,
                    background,
                }
            }
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
            (_, CommandCode::CONDITONAL_BRANCH) => Command::parse_conditional_branch(event_command)
                .context("failed to parse CONDITONAL_BRANCH command")?,
            (_, CommandCode::EXIT_EVENT_PROCESSING) => {
                ensure!(event_command.parameters.is_empty());
                Command::ExitEventProcessing
            }
            (_, CommandCode::COMMON_EVENT) => {
                ensure!(event_command.parameters.len() == 1);
                let id = event_command.parameters[0]
                    .as_i64()
                    .and_then(|value| u32::try_from(value).ok())
                    .context("`id` is not a `u32`")?;

                Command::CommonEvent { id }
            }
            (_, CommandCode::CONTROL_SWITCHES) => {
                ensure!(event_command.parameters.len() == 3);

                let start_id = event_command.parameters[0]
                    .as_i64()
                    .and_then(|value| u32::try_from(value).ok())
                    .context("`start_id` is not a `u32`")?;
                let end_id = event_command.parameters[1]
                    .as_i64()
                    .and_then(|value| u32::try_from(value).ok())
                    .context("`end_id` is not a `u32`")?;
                let value = event_command.parameters[2]
                    .as_i64()
                    .and_then(|value| u32::try_from(value).ok())
                    .context("`value` is not a `u32`")?;
                ensure!(value <= 1);
                let value = value == 0;

                Command::ControlSwitches {
                    start_id,
                    end_id,
                    value,
                }
            }
            (_, CommandCode::CONTROL_VARIABLES) => {
                ensure!(event_command.parameters.len() >= 4);
                let start_variable_id = event_command.parameters[0]
                    .as_i64()
                    .and_then(|value| u32::try_from(value).ok())
                    .context("`start_variable_id` is not a `u32`")?;
                let end_variable_id = event_command.parameters[1]
                    .as_i64()
                    .and_then(|value| u32::try_from(value).ok())
                    .context("`end_variable_id` is not a `u32`")?;
                let operate_variable_operation = event_command.parameters[2]
                    .as_i64()
                    .and_then(|value| u8::try_from(value).ok())
                    .context("`control_variables_operation` is not a `u8`")?;
                let operate_variable_operation =
                    OperateVariableOperation::from_u8(operate_variable_operation)?;
                let control_variables_operation = event_command.parameters[3]
                    .as_i64()
                    .and_then(|value| u8::try_from(value).ok())
                    .context("`control_variables_operation` is not a `u8`")?;
                let control_variables_operation =
                    ControlVariablesOperation::from_u8(control_variables_operation)?;

                let value = match control_variables_operation {
                    ControlVariablesOperation::Const => {
                        ensure!(event_command.parameters.len() == 5);

                        let value = event_command.parameters[4]
                            .as_i64()
                            .and_then(|value| u32::try_from(value).ok())
                            .context("`value` is not a `u32`")?;

                        ControlVariablesValue::Constant { value }
                    }
                    ControlVariablesOperation::Var => {
                        ensure!(event_command.parameters.len() == 5);

                        let id = event_command.parameters[4]
                            .as_i64()
                            .and_then(|value| u32::try_from(value).ok())
                            .context("`id` is not a `u32`")?;

                        ControlVariablesValue::Variable { id }
                    }
                    ControlVariablesOperation::Random => {
                        ensure!(event_command.parameters.len() == 6);
                        let start = event_command.parameters[4]
                            .as_i64()
                            .and_then(|value| u32::try_from(value).ok())
                            .context("`start` is not a `u32`")?;
                        let stop = event_command.parameters[5]
                            .as_i64()
                            .and_then(|value| u32::try_from(value).ok())
                            .context("`stop` is not a `u32`")?;

                        ControlVariablesValue::Random { start, stop }
                    }
                    ControlVariablesOperation::GameData => {
                        ensure!(event_command.parameters.len() == 7);
                        let kind = event_command.parameters[4]
                            .as_i64()
                            .and_then(|value| u8::try_from(value).ok())
                            .context("`kind` is not a `u8`")?;
                        let kind = GameDataOperandKind::from_u8(kind)?;
                        let param1 = event_command.parameters[5]
                            .as_i64()
                            .and_then(|value| u32::try_from(value).ok())
                            .context("`param1` is not a `u32`")?;
                        let param2 = event_command.parameters[6]
                            .as_i64()
                            .and_then(|value| u32::try_from(value).ok())
                            .context("`param2` is not a `u32`")?;

                        let inner = match kind {
                            GameDataOperandKind::Item => {
                                let item_id = param1;

                                ControlVariablesValueGameData::NumItems { item_id }
                            }
                            GameDataOperandKind::Actor => {
                                let actor_id = param1;
                                let check =
                                    u8::try_from(param2).context("`check` is not a `u8`")?;
                                let check = GameDataOperandKindActorCheck::from_u8(check)?;

                                match check {
                                    GameDataOperandKindActorCheck::Level => {
                                        ControlVariablesValueGameData::ActorLevel { actor_id }
                                    }
                                    _ => bail!(
                                        "GameDataOperandKindActorCheck {check:?} is not supported"
                                    ),
                                }
                            }
                            GameDataOperandKind::Other => {
                                let check =
                                    u8::try_from(param1).context("`check` is not a `u8`")?;
                                let check = GameDataOperandKindOtherCheck::from_u8(check)?;

                                match check {
                                    GameDataOperandKindOtherCheck::MapId => {
                                        ControlVariablesValueGameData::MapId
                                    }
                                    GameDataOperandKindOtherCheck::Gold => {
                                        ControlVariablesValueGameData::Gold
                                    }
                                    _ => bail!(
                                        "GameDataOperandKindOtherCheck {check:?} is not supported"
                                    ),
                                }
                            }
                            _ => {
                                bail!("GameDataOperandKind {kind:?} is not supported")
                            }
                        };

                        ControlVariablesValue::GameData(inner)
                    }
                    _ => {
                        let name = "ControlVariablesOperation";
                        bail!("{name} {control_variables_operation:?} is not supported")
                    }
                };

                Command::ControlVariables {
                    start_variable_id,
                    end_variable_id,
                    operation: operate_variable_operation,
                    value,
                }
            }
            (_, CommandCode::CONTROL_SELF_SWITCH) => {
                ensure!(event_command.parameters.len() == 2);
                let key = event_command.parameters[0]
                    .as_str()
                    .context("`key` is not a `str`")?
                    .to_string();
                let value = event_command.parameters[1]
                    .as_i64()
                    .and_then(|value| u8::try_from(value).ok())
                    .context("`value` is not a `u8`")?;
                ensure!(value <= 1);
                let value = value == 0;

                Command::ControlSelfSwitch { key, value }
            }
            (_, CommandCode::CHANGE_ITEMS) => {
                ensure!(event_command.parameters.len() == 4);
                let item_id = event_command.parameters[0]
                    .as_i64()
                    .and_then(|value| u32::try_from(value).ok())
                    .context("`item_id` is not a `u32`")?;
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

                Command::ChangeItems {
                    item_id,
                    is_add,
                    value,
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
            (_, CommandCode::TRANSFER_PLAYER) => Command::parse_transfer_player(event_command)
                .context("failed to parse TRANSFER_PLAYER command")?,
            (_, CommandCode::SET_MOVEMENT_ROUTE) => {
                ensure!(event_command.parameters.len() == 2);
                let character_id = event_command.parameters[0]
                    .as_i64()
                    .and_then(|value| i32::try_from(value).ok())
                    .context("`value` is not an `i32`")?;
                let route: rpgmv_types::MoveRoute =
                    serde_json::from_value(event_command.parameters[1].clone())
                        .context("invalid `route` parameter")?;

                move_command_index = 0;

                Command::SetMovementRoute {
                    character_id,
                    route,
                }
            }
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
            (_, CommandCode::SHOW_BALLOON_ICON) => {
                ensure!(event_command.parameters.len() == 3);
                let character_id = event_command.parameters[0]
                    .as_i64()
                    .and_then(|value| i32::try_from(value).ok())
                    .context("`character_id` is not a `i32`")?;
                let balloon_id = event_command.parameters[1]
                    .as_i64()
                    .and_then(|value| u32::try_from(value).ok())
                    .context("`balloon_id` is not a `u32`")?;
                let wait = event_command.parameters[2]
                    .as_bool()
                    .context("`wait` is not a `bool`")?;

                Command::ShowBalloonIcon {
                    character_id,
                    balloon_id,
                    wait,
                }
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
            (_, CommandCode::FADEOUT_SCREEN) => {
                ensure!(event_command.parameters.is_empty());
                Command::FadeoutScreen
            }
            (_, CommandCode::FADEIN_SCREEN) => {
                ensure!(event_command.parameters.is_empty());
                Command::FadeinScreen
            }
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
            (_, CommandCode::SHAKE_SCREEN) => {
                ensure!(event_command.parameters.len() == 4);
                let power = event_command.parameters[0]
                    .as_i64()
                    .and_then(|value| u32::try_from(value).ok())
                    .context("`power` is not a `u32`")?;
                let speed = event_command.parameters[1]
                    .as_i64()
                    .and_then(|value| u32::try_from(value).ok())
                    .context("`speed` is not a `u32`")?;
                let duration = event_command.parameters[2]
                    .as_i64()
                    .and_then(|value| u32::try_from(value).ok())
                    .context("`duration` is not a `u32`")?;
                let wait = event_command.parameters[3]
                    .as_bool()
                    .context("`wait` is not a `bool`")?;

                Command::ShakeScreen {
                    power,
                    speed,
                    duration,
                    wait,
                }
            }
            (_, CommandCode::WAIT) => {
                ensure!(event_command.parameters.len() == 1);
                let duration = event_command.parameters[0]
                    .as_i64()
                    .and_then(|value| u32::try_from(value).ok())
                    .context("`duration` is not a `u32`")?;

                Command::Wait { duration }
            }
            (_, CommandCode::SHOW_PICTURE) => Command::parse_show_picture(event_command)
                .context("failed to parse SHOW_PICTURE command")?,
            (_, CommandCode::ERASE_PICTURE) => {
                ensure!(event_command.parameters.len() == 1);

                let picture_id = event_command.parameters[0]
                    .as_i64()
                    .and_then(|value| u32::try_from(value).ok())
                    .context("`picture_id` is not a `u32`")?;

                Command::ErasePicture { picture_id }
            }
            (_, CommandCode::PLAY_BGM) => {
                ensure!(event_command.parameters.len() == 1);
                let audio: rpgmv_types::AudioFile =
                    serde_json::from_value(event_command.parameters[0].clone())
                        .context("invalid `audio` parameter")?;

                Command::PlayBgm { audio }
            }
            (_, CommandCode::FADEOUT_BGM) => {
                ensure!(event_command.parameters.len() == 1);

                let duration = event_command.parameters[0]
                    .as_i64()
                    .and_then(|value| u32::try_from(value).ok())
                    .context("`duration` is not a `u32`")?;

                Command::FadeoutBgm { duration }
            }
            (_, CommandCode::PLAY_SE) => {
                ensure!(event_command.parameters.len() == 1);
                let audio: rpgmv_types::AudioFile =
                    serde_json::from_value(event_command.parameters[0].clone())
                        .context("invalid `audio` parameter")?;

                Command::PlaySe { audio }
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
            (_, CommandCode::SCRIPT) => {
                ensure!(event_command.parameters.len() == 1);
                let line = event_command.parameters[0]
                    .as_str()
                    .context("`line` is not a string")?
                    .to_string();

                Command::Script { lines: vec![line] }
            }
            (_, CommandCode::WHEN) => {
                ensure!(event_command.parameters.len() == 2);
                let choice_index = event_command.parameters[0]
                    .as_i64()
                    .and_then(|value| u32::try_from(value).ok())
                    .context("`choice_index` is not a `u32`")?;
                let choice_name = event_command.parameters[1]
                    .as_str()
                    .context("`choice_name` is not a string")?
                    .to_string();

                Command::When {
                    choice_index,
                    choice_name,
                }
            }
            (_, CommandCode::WHEN_END) => {
                ensure!(event_command.parameters.is_empty());
                Command::WhenEnd
            }
            (_, CommandCode::ELSE) => {
                ensure!(event_command.parameters.is_empty());
                Command::Else
            }
            (_, CommandCode::CONDITONAL_BRANCH_END) => {
                ensure!(event_command.parameters.is_empty());
                Command::ConditionalBranchEnd
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
