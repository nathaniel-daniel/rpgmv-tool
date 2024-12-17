use super::Command;
use super::MaybeRef;
use anyhow::bail;
use anyhow::ensure;
use anyhow::Context;

#[derive(Debug, Copy, Clone)]
pub(super) enum ConditionalBranchKind {
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
pub(super) enum ConditionalBranchKindEnemyCheck {
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

impl Command {
    pub(super) fn parse_conditional_branch(
        event_command: &rpgmv_types::EventCommand,
    ) -> anyhow::Result<Self> {
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
            ConditionalBranchKind::SelfSwitch => {
                ensure!(event_command.parameters.len() == 3);

                let name = event_command.parameters[1]
                    .as_str()
                    .context("`name` is not a `String`")?
                    .to_string();
                let check_true = event_command.parameters[2]
                    .as_i64()
                    .and_then(|value| u8::try_from(value).ok())
                    .context("`check_true` is not a `u32`")?;
                ensure!(check_true <= 1);
                let check_true = check_true == 0;

                ConditionalBranchCommand::SelfSwitch { name, check_true }
            }
            ConditionalBranchKind::Timer => {
                ensure!(event_command.parameters.len() == 3);

                let value = event_command.parameters[1]
                    .as_i64()
                    .and_then(|value| u32::try_from(value).ok())
                    .context("`value` is not a `u32`")?;
                let is_gte = event_command.parameters[2]
                    .as_i64()
                    .and_then(|value| u8::try_from(value).ok())
                    .context("`is_gte` is not a `u8`")?;
                ensure!(is_gte <= 1);
                let is_gte = is_gte == 0;

                ConditionalBranchCommand::Timer { value, is_gte }
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
                    ConditionalBranchKindActorCheck::Skill => {
                        ensure!(event_command.parameters.len() == 4);

                        let skill_id = event_command.parameters[3]
                            .as_i64()
                            .and_then(|value| u32::try_from(value).ok())
                            .context("`skill_id` is not a `u32`")?;

                        ConditionalBranchCommand::ActorSkill { actor_id, skill_id }
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
    SelfSwitch {
        name: String,
        check_true: bool,
    },
    Timer {
        value: u32,
        is_gte: bool,
    },
    ActorInParty {
        actor_id: u32,
    },
    ActorSkill {
        actor_id: u32,
        skill_id: u32,
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
