use anyhow::bail;
use anyhow::ensure;
use anyhow::Context;

/// A command code
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct CommandCode(u32);

impl CommandCode {
    /// This is likely related to move routes somehow,
    /// Like how the TEXT_DATA command extends the SHOW_TEXT command.
    /// However, I can't find the implementation of this instruction.
    const UNKNOWN_505: Self = Self(505);

    const NOP: Self = Self(0);

    const SHOW_TEXT: Self = Self(101);
    const SHOW_CHOICES: Self = Self(102);

    const CONDITONAL_BRANCH: Self = Self(111);

    const COMMON_EVENT: Self = Self(117);

    const CONTROL_SWITCHES: Self = Self(121);
    const CONTROL_VARIABLES: Self = Self(122);

    const CHANGE_ITEMS: Self = Self(126);

    const CHANGE_SAVE_ACCESS: Self = Self(134);

    const TRANSFER_PLAYER: Self = Self(201);

    const SET_MOVEMENT_ROUTE: Self = Self(205);

    const CHANGE_TRANSPARENCY: Self = Self(211);
    const SHOW_ANIMATION: Self = Self(212);
    const SHOW_BALLOON_ICON: Self = Self(213);

    const FADEOUT_SCREEN: Self = Self(221);
    const FADEIN_SCREEN: Self = Self(222);
    const TINT_SCREEN: Self = Self(223);
    const FLASH_SCREEN: Self = Self(224);

    const WAIT: Self = Self(230);
    const SHOW_PICTURE: Self = Self(231);
    const MOVE_PICTURE: Self = Self(232);

    const ERASE_PICTURE: Self = Self(235);

    const PLAY_SE: Self = Self(250);

    const CHANGE_SKILL: Self = Self(318);

    const TEXT_DATA: Self = Self(401);
    const WHEN: Self = Self(402);

    /// I think this is an end for the WHEN block.
    /// I can't be sure as the game doesn't actually care if this exists;
    /// it just ignores it, only taking into account indents.
    const WHEN_END: Self = Self(404);

    const ELSE: Self = Self(411);
    /// I think this is an end for the CONDITONAL_BRANCH block.
    /// I can't be sure as the game doesn't actually care if this exists;
    /// it just ignores it, only taking into account indents.
    const CONDITONAL_BRANCH_END: Self = Self(412);
}

impl std::fmt::Debug for CommandCode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Self::UNKNOWN_505 => write!(f, "UNKNOWN_505"),
            Self::NOP => write!(f, "NOP"),
            Self::SHOW_TEXT => write!(f, "SHOW_TEXT"),
            Self::SHOW_CHOICES => write!(f, "SHOW_CHOICES"),
            Self::CONDITONAL_BRANCH => write!(f, "CONDITONAL_BRANCH"),
            Self::COMMON_EVENT => write!(f, "COMMON_EVENT"),
            Self::CONTROL_SWITCHES => write!(f, "CONTROL_SWITCHES"),
            Self::CONTROL_VARIABLES => write!(f, "CONTROL_VARIABLES"),
            Self::CHANGE_ITEMS => write!(f, "CHANGE_ITEMS"),
            Self::CHANGE_SAVE_ACCESS => write!(f, "CHANGE_SAVE_ACCESS"),
            Self::TRANSFER_PLAYER => write!(f, "TRANSFER_PLAYER"),
            Self::SET_MOVEMENT_ROUTE => write!(f, "SET_MOVEMENT_ROUTE"),
            Self::CHANGE_TRANSPARENCY => write!(f, "CHANGE_TRANSPARENCY"),
            Self::SHOW_ANIMATION => write!(f, "SHOW_ANIMATION"),
            Self::SHOW_BALLOON_ICON => write!(f, "SHOW_BALLOON_ICON"),
            Self::FADEOUT_SCREEN => write!(f, "FADEOUT_SCREEN"),
            Self::FADEIN_SCREEN => write!(f, "FADEIN_SCREEN"),
            Self::TINT_SCREEN => write!(f, "TINT_SCREEN"),
            Self::FLASH_SCREEN => write!(f, "FLASH_SCREEN"),
            Self::WAIT => write!(f, "WAIT"),
            Self::SHOW_PICTURE => write!(f, "SHOW_PICTURE"),
            Self::MOVE_PICTURE => write!(f, "MOVE_PICTURE"),
            Self::ERASE_PICTURE => write!(f, "ERASE_PICTURE"),
            Self::PLAY_SE => write!(f, "PLAY_SE"),
            Self::CHANGE_SKILL => write!(f, "CHANGE_SKILL"),
            Self::TEXT_DATA => write!(f, "TEXT_DATA"),
            Self::WHEN => write!(f, "WHEN"),
            Self::WHEN_END => write!(f, "WHEN_END"),
            Self::ELSE => write!(f, "ELSE"),
            Self::CONDITONAL_BRANCH_END => write!(f, "CONDITONAL_BRANCH_END"),
            _ => write!(f, "Unknown({})", self.0),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum ConditionalBranchKind {
    Switch = 0,
    Variable = 1,

    Actor = 4,

    Gold = 7,

    Script = 12,
}

impl ConditionalBranchKind {
    /// Get this from a u8.
    pub fn from_u8(value: u8) -> anyhow::Result<Self> {
        match value {
            0 => Ok(Self::Switch),
            1 => Ok(Self::Variable),
            4 => Ok(Self::Actor),
            7 => Ok(Self::Gold),
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

#[derive(Debug, Copy, Clone)]
pub enum ConditionalBranchVariableOperation {
    EqualTo = 0,
    Gte = 1,
    Lte = 2,
    Gt = 3,
    Lt = 4,
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
pub enum ControlVariablesOperation {
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
pub enum OperateVariableOperation {
    Set = 0,
    Add = 1,
    Sub = 2,
    Mul = 3,
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
        default_type: u32,
        position_type: u32,
        background: u32,
    },
    ConditionalBranch(ConditionalBranchCommand),
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
    ChangeItems {
        item_id: u32,
        is_add: bool,
        value: MaybeRef<u32>,
    },
    ChangeTransparency {
        set_transparent: bool,
    },
    ShowBalloonIcon {
        character_id: i32,
        balloon_id: u32,
        wait: bool,
    },
    FadeoutScreen,
    FadeinScreen,
    FlashScreen {
        color: [u8; 4],
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
        x: MaybeRef<u32>,
        y: MaybeRef<u32>,
        scale_x: u32,
        scale_y: u32,
        opacity: u8,
        blend_mode: u8,
    },
    ErasePicture {
        picture_id: u32,
    },
    PlaySe {
        audio: rpgmv_types::AudioFile,
    },
    ChangeSkill {
        actor_id: MaybeRef<u32>,
        is_learn_skill: bool,
        skill_id: u32,
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
}

#[derive(Debug)]
pub enum ControlVariablesValue {
    Constant { value: u32 },
    Variable { id: u32 },
    Random { start: u32, stop: u32 },
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

    for event_command in list.iter() {
        let command_code = CommandCode(event_command.code);

        let last_command = ret.last_mut().map(|(_code, command)| command);
        let command = match (last_command, command_code) {
            (Some(Command::ShowText { lines, .. }), CommandCode::TEXT_DATA) => {
                ensure!(event_command.parameters.len() == 1);
                let line = event_command.parameters[0]
                    .as_str()
                    .context("line is not a string")?
                    .to_string();

                lines.push(line);

                continue;
            }
            (_, CommandCode::NOP) => {
                ensure!(event_command.parameters.is_empty());

                Command::Nop
            }
            (_, CommandCode::SHOW_TEXT) => {
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

                Command::ShowText {
                    face_name,
                    face_index,
                    background,
                    position_type,
                    lines: Vec::new(),
                }
            }
            (_, CommandCode::SHOW_CHOICES) => {
                ensure!(event_command.parameters.len() == 5);

                let choices = serde_json::from_value(event_command.parameters[0].clone())?;
                let cancel_type = event_command.parameters[1]
                    .as_i64()
                    .and_then(|value| i32::try_from(value).ok())
                    .context("`cancel_type` is not an `i32`")?;
                let default_type = event_command.parameters[2]
                    .as_i64()
                    .and_then(|value| u32::try_from(value).ok())
                    .context("`default_type` is not a `u32`")?;
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
            (_, CommandCode::CONDITONAL_BRANCH) => {
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
                    _ => bail!("ConditionalBranchKind {kind:?} is not supported"),
                };

                Command::ConditionalBranch(inner)
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
            (_, CommandCode::FADEOUT_SCREEN) => {
                ensure!(event_command.parameters.is_empty());
                Command::FadeoutScreen
            }
            (_, CommandCode::FADEIN_SCREEN) => {
                ensure!(event_command.parameters.is_empty());
                Command::FadeinScreen
            }
            (_, CommandCode::FLASH_SCREEN) => {
                ensure!(event_command.parameters.len() == 3);
                let color: [u8; 4] = serde_json::from_value(event_command.parameters[0].clone())?;
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
            (_, CommandCode::WAIT) => {
                ensure!(event_command.parameters.len() == 1);
                let duration = event_command.parameters[0]
                    .as_i64()
                    .and_then(|value| u32::try_from(value).ok())
                    .context("`duration` is not a `u32`")?;

                Command::Wait { duration }
            }
            (_, CommandCode::SHOW_PICTURE) => {
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
                    .and_then(|value| u32::try_from(value).ok())
                    .context("`x` is not a `u32`")?;
                let y = event_command.parameters[5]
                    .as_i64()
                    .and_then(|value| u32::try_from(value).ok())
                    .context("`y` is not a `u32`")?;
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
                    (MaybeRef::Constant(x), MaybeRef::Constant(y))
                } else {
                    (MaybeRef::Ref(x), MaybeRef::Ref(y))
                };

                Command::ShowPicture {
                    picture_id,
                    picture_name,
                    origin,
                    x,
                    y,
                    scale_x,
                    scale_y,
                    opacity,
                    blend_mode,
                }
            }
            (_, CommandCode::ERASE_PICTURE) => {
                ensure!(event_command.parameters.len() == 1);

                let picture_id = event_command.parameters[0]
                    .as_i64()
                    .and_then(|value| u32::try_from(value).ok())
                    .context("`picture_id` is not a `u32`")?;

                Command::ErasePicture { picture_id }
            }
            (_, CommandCode::PLAY_SE) => {
                ensure!(event_command.parameters.len() == 1);
                let audio = serde_json::from_value(event_command.parameters[0].clone())?;

                Command::PlaySe { audio }
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
