use super::Command;
use anyhow::bail;
use anyhow::ensure;
use anyhow::Context;

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
    Hp = 2,
    Mp = 3,
}

impl GameDataOperandKindActorCheck {
    /// Get this from a u8.
    pub fn from_u8(value: u8) -> anyhow::Result<Self> {
        match value {
            0 => Ok(Self::Level),
            1 => Ok(Self::Exp),
            2 => Ok(Self::Hp),
            3 => Ok(Self::Mp),
            _ => bail!("{value} is not a valid GameDataOperandKindActorCheck"),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum GameDataOperandKindCharacterCheck {
    MapX = 0,
    MapY = 1,
    Direction = 2,
    ScreenX = 3,
    ScreenY = 4,
}

impl GameDataOperandKindCharacterCheck {
    /// Get this from a u8.
    pub fn from_u8(value: u8) -> anyhow::Result<Self> {
        match value {
            0 => Ok(Self::MapX),
            1 => Ok(Self::MapY),
            2 => Ok(Self::Direction),
            3 => Ok(Self::ScreenX),
            4 => Ok(Self::ScreenY),
            _ => bail!("{value} is not a valid GameDataOperandKindCharacterCheck"),
        }
    }
}

#[derive(Debug)]
pub enum ControlVariablesValue {
    Constant { value: i32 },
    Variable { id: u32 },
    Random { start: i32, stop: i32 },
    GameData(ControlVariablesValueGameData),
}

#[derive(Debug)]
pub enum ControlVariablesValueGameData {
    NumItems { item_id: u32 },
    ActorLevel { actor_id: u32 },
    ActorMp { actor_id: u32 },
    CharacterMapX { character_id: i32 },
    CharacterMapY { character_id: i32 },
    MapId,
    Gold,
    Steps,
}

impl Command {
    pub(super) fn parse_control_variables(
        event_command: &rpgmv_types::EventCommand,
    ) -> anyhow::Result<Self> {
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
                    .and_then(|value| i32::try_from(value).ok())
                    .context("`value` is not an `i32`")?;

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
                    .and_then(|value| i32::try_from(value).ok())
                    .context("`start` is not an `i32`")?;
                let stop = event_command.parameters[5]
                    .as_i64()
                    .and_then(|value| i32::try_from(value).ok())
                    .context("`stop` is not an `i32`")?;

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
                    .and_then(|value| i32::try_from(value).ok())
                    .context("`param1` is not an `i32`")?;
                let param2 = event_command.parameters[6]
                    .as_i64()
                    .and_then(|value| u32::try_from(value).ok())
                    .context("`param2` is not a `u32`")?;

                let inner = match kind {
                    GameDataOperandKind::Item => {
                        let item_id = u32::try_from(param1).context("`item_id` is not a `u32`")?;

                        ControlVariablesValueGameData::NumItems { item_id }
                    }
                    GameDataOperandKind::Actor => {
                        let actor_id =
                            u32::try_from(param1).context("`actor_id` is not a `u32`")?;
                        let check = u8::try_from(param2).context("`check` is not a `u8`")?;
                        let check = GameDataOperandKindActorCheck::from_u8(check)?;

                        match check {
                            GameDataOperandKindActorCheck::Level => {
                                ControlVariablesValueGameData::ActorLevel { actor_id }
                            }
                            GameDataOperandKindActorCheck::Mp => {
                                ControlVariablesValueGameData::ActorMp { actor_id }
                            }
                            _ => bail!("GameDataOperandKindActorCheck {check:?} is not supported"),
                        }
                    }
                    GameDataOperandKind::Character => {
                        let character_id = param1;
                        let check = u8::try_from(param2).context("`check` is not a `u8`")?;
                        let check = GameDataOperandKindCharacterCheck::from_u8(check)?;

                        match check {
                            GameDataOperandKindCharacterCheck::MapX => {
                                ControlVariablesValueGameData::CharacterMapX { character_id }
                            }
                            GameDataOperandKindCharacterCheck::MapY => {
                                ControlVariablesValueGameData::CharacterMapY { character_id }
                            }
                            _ => bail!(
                                "GameDataOperandKindCharacterCheck {check:?} is not supported"
                            ),
                        }
                    }
                    GameDataOperandKind::Other => {
                        let check = u8::try_from(param1).context("`check` is not a `u8`")?;
                        let check = GameDataOperandKindOtherCheck::from_u8(check)?;

                        match check {
                            GameDataOperandKindOtherCheck::MapId => {
                                ControlVariablesValueGameData::MapId
                            }
                            GameDataOperandKindOtherCheck::Gold => {
                                ControlVariablesValueGameData::Gold
                            }
                            GameDataOperandKindOtherCheck::Steps => {
                                ControlVariablesValueGameData::Steps
                            }
                            _ => bail!("GameDataOperandKindOtherCheck {check:?} is not supported"),
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

        Ok(Command::ControlVariables {
            start_variable_id,
            end_variable_id,
            operation: operate_variable_operation,
            value,
        })
    }
}
