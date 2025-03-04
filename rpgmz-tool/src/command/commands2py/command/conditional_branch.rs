use super::Command;
use super::IntBool;
use super::MaybeRef;
use super::ParamReader;
use anyhow::bail;

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
        event_command: &rpgmz_types::EventCommand,
    ) -> anyhow::Result<Self> {
        let reader = ParamReader::new(event_command);
        reader.ensure_len_is_at_least(1)?;

        let kind = reader.read_at(0, "kind")?;
        let kind = ConditionalBranchKind::from_u8(kind)?;

        let inner = match kind {
            ConditionalBranchKind::Switch => {
                reader.ensure_len_is(3)?;

                let id = reader.read_at(1, "id")?;
                let IntBool(check_true) = reader.read_at(2, "check_true")?;

                ConditionalBranchCommand::Switch { id, check_true }
            }
            ConditionalBranchKind::Variable => {
                reader.ensure_len_is(5)?;

                let lhs_id = reader.read_at(1, "lhs_id")?;
                let IntBool(is_constant) = reader.read_at(2, "is_constant")?;
                let rhs_id = reader.read_at(3, "rhs_id")?;

                let rhs_id = if is_constant {
                    MaybeRef::Constant(rhs_id)
                } else {
                    MaybeRef::Ref(rhs_id)
                };

                let operation = reader.read_at(4, "operation")?;
                let operation = ConditionalBranchVariableOperation::from_u8(operation)?;

                ConditionalBranchCommand::Variable {
                    lhs_id,
                    rhs_id,
                    operation,
                }
            }
            ConditionalBranchKind::SelfSwitch => {
                reader.ensure_len_is(3)?;

                let name = reader.read_at(1, "name")?;
                let IntBool(check_true) = reader.read_at(2, "check_true")?;

                ConditionalBranchCommand::SelfSwitch { name, check_true }
            }
            ConditionalBranchKind::Timer => {
                reader.ensure_len_is(3)?;

                let value = reader.read_at(1, "value")?;
                let IntBool(is_gte) = reader.read_at(2, "is_gte")?;

                ConditionalBranchCommand::Timer { value, is_gte }
            }
            ConditionalBranchKind::Actor => {
                reader.ensure_len_is_at_least(3)?;

                let actor_id = reader.read_at(1, "actor_id")?;
                let check = reader.read_at(2, "check")?;
                let check = ConditionalBranchKindActorCheck::from_u8(check)?;

                match check {
                    ConditionalBranchKindActorCheck::InParty => {
                        reader.ensure_len_is(3)?;

                        ConditionalBranchCommand::ActorInParty { actor_id }
                    }
                    ConditionalBranchKindActorCheck::Skill => {
                        reader.ensure_len_is(4)?;

                        let skill_id = reader.read_at(3, "skill_id")?;

                        ConditionalBranchCommand::ActorSkill { actor_id, skill_id }
                    }
                    ConditionalBranchKindActorCheck::Armor => {
                        reader.ensure_len_is(4)?;

                        let armor_id = reader.read_at(3, "armor_id")?;

                        ConditionalBranchCommand::ActorArmor { actor_id, armor_id }
                    }
                    ConditionalBranchKindActorCheck::State => {
                        reader.ensure_len_is(4)?;

                        let state_id = reader.read_at(3, "state_id")?;

                        ConditionalBranchCommand::ActorState { actor_id, state_id }
                    }
                    _ => {
                        bail!("ConditionalBranchKindActorCheck {check:?} is not supported")
                    }
                }
            }
            ConditionalBranchKind::Enemy => {
                reader.ensure_len_is_at_least(3)?;

                let enemy_index = reader.read_at(1, "enemy_index")?;
                let check = reader.read_at(2, "check")?;
                let check = ConditionalBranchKindEnemyCheck::from_u8(check)?;

                match check {
                    ConditionalBranchKindEnemyCheck::State => {
                        reader.ensure_len_is(4)?;

                        let state_id = reader.read_at(2, "state_id")?;

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
                reader.ensure_len_is(3)?;

                let character_id = reader.read_at(1, "character_id")?;
                let direction = reader.read_at(2, "direction")?;

                ConditionalBranchCommand::Character {
                    character_id,
                    direction,
                }
            }
            ConditionalBranchKind::Gold => {
                reader.ensure_len_is(3)?;

                let value = reader.read_at(1, "value")?;
                let check = reader.read_at(2, "check")?;
                let check = ConditionalBranchKindGoldCheck::from_u8(check)?;

                ConditionalBranchCommand::Gold { value, check }
            }
            ConditionalBranchKind::Item => {
                reader.ensure_len_is(2)?;

                let item_id = reader.read_at(1, "item_id")?;

                ConditionalBranchCommand::Item { item_id }
            }
            ConditionalBranchKind::Button => {
                reader.ensure_len_is(2)?;

                let key_name = reader.read_at(1, "key_name")?;

                ConditionalBranchCommand::Button { key_name }
            }
            ConditionalBranchKind::Script => {
                reader.ensure_len_is(2)?;

                let value = reader.read_at(1, "value")?;

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
    Button {
        key_name: String,
    },
    Script {
        value: String,
    },
}
