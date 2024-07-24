mod config;

use self::config::Config;
use anyhow::bail;
use anyhow::ensure;
use anyhow::Context;
use std::fmt::Write as _;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

#[derive(Debug, argh::FromArgs)]
#[argh(
    subcommand,
    name = "commands2py",
    description = "a tool to \"decompile\" scripts to Python for easier inspection"
)]
pub struct Options {
    #[argh(
        option,
        long = "input",
        short = 'i',
        description = "the path to the input file"
    )]
    input: PathBuf,

    #[argh(option, long = "event-id", description = "the event id to convert")]
    event_id: u32,

    #[argh(option, long = "event-page", description = "the event page to convert")]
    event_page: Option<u16>,

    #[argh(
        option,
        long = "config",
        short = 'c',
        description = "the path to the config to use"
    )]
    config: Option<PathBuf>,

    #[argh(
        option,
        long = "output",
        short = 'o',
        description = "the path to the output file",
        default = "PathBuf::from(\"out.py\")"
    )]
    output: PathBuf,
}

pub fn exec(options: Options) -> anyhow::Result<()> {
    let config = match options.config {
        Some(config) => Config::from_path(&config)
            .with_context(|| format!("failed to load config from \"{}\"", config.display()))?,
        None => Config::default(),
    };

    let input_str = std::fs::read_to_string(&options.input)
        .with_context(|| format!("failed to read \"{}\"", options.input.display()))?;
    let input: rpgmv_types::Map = serde_json::from_str(&input_str)
        .with_context(|| format!("failed to read \"{}\"", options.input.display()))?;

    let event = usize::try_from(options.event_id)
        .ok()
        .and_then(|event_id| input.events.get(event_id)?.as_ref())
        .with_context(|| format!("no event with id {}", options.event_id))?;
    ensure!(event.id == options.event_id);

    let event_page_index = match options.event_page {
        Some(event_page) => event_page,
        None if event.pages.len() == 1 => 0,
        None => bail!("found multiple event pages. specify which one with --event-page flag."),
    };
    let event_page = event
        .pages
        .get(usize::from(event_page_index))
        .with_context(|| format!("no event page with index {event_page_index}"))?;

    let commands = parse_event_command_list(&event_page.list)?;

    let mut python = String::new();
    for (indent, command) in commands {
        write_indent(&mut python, indent);

        match command {
            Command::Nop => {}
            Command::ShowText {
                face_name,
                face_index,
                background,
                position_type,
                lines,
            } => {
                writeln!(&mut python, "ShowText(")?;

                write_indent(&mut python, indent + 1);
                writeln!(&mut python, "face_name='{face_name}',")?;

                write_indent(&mut python, indent + 1);
                writeln!(&mut python, "face_index={face_index},")?;

                write_indent(&mut python, indent + 1);
                writeln!(&mut python, "background={background},")?;

                write_indent(&mut python, indent + 1);
                writeln!(&mut python, "position_type={position_type},")?;

                write_indent(&mut python, indent + 1);
                writeln!(&mut python, "lines=[")?;

                for line in lines {
                    let line = line.replace('\'', "\\'");

                    write_indent(&mut python, indent + 2);
                    writeln!(&mut python, "'{line}',")?;
                }

                write_indent(&mut python, indent + 1);
                writeln!(&mut python, "],")?;

                write_indent(&mut python, indent);
                writeln!(&mut python, ")")?;
            }
            Command::ConditionalBranch(command) => {
                write!(&mut python, "if ")?;
                match command {
                    ConditionalBranchCommand::Variable {
                        lhs_id,
                        rhs_id,
                        operation,
                    } => {
                        let lhs = config.get_variable_name(lhs_id);
                        let rhs = match rhs_id {
                            MaybeRef::Constant(value) => value.to_string(),
                            MaybeRef::Ref(id) => config.get_variable_name(id),
                        };
                        let operation = operation.as_str();

                        writeln!(&mut python, "{lhs} {operation} {rhs}:")?;
                    }
                }
            }
            Command::ControlSwitches {
                start_id,
                end_id,
                value,
            } => {
                let mut iter = (start_id..(end_id + 1)).peekable();
                let value = stringify_bool(value);

                while let Some(id) = iter.next() {
                    let name = config.get_switch_name(id);

                    writeln!(&mut python, "{name} = {value}")?;
                    if iter.peek().is_some() {
                        write_indent(&mut python, indent);
                    }
                }
            }
            Command::ChangeTransparency { set_transparent } => writeln!(
                &mut python,
                "ChangeTransparency(set_transparent={})",
                stringify_bool(set_transparent)
            )?,
            Command::FadeoutScreen => writeln!(&mut python, "FadeoutScreen()")?,
            Command::Wait { duration } => writeln!(&mut python, "Wait(duration={duration})")?,
            Command::ConditionalBranchEnd => {
                // Trust indents over branch ends
            }
            Command::Unknown { code, parameters } => {
                writeln!(
                    &mut python,
                    "# Unknown Command Code {code:?}, parameters: {parameters:?}"
                )?;
            }
        }
    }

    let output_temp = nd_util::with_push_extension(&options.output, "tmp");
    let mut output_file = File::create(&output_temp)
        .with_context(|| format!("failed to open \"{}\"", output_temp.display()))?;
    output_file.write_all(python.as_bytes())?;
    output_file.flush()?;
    output_file.sync_all()?;
    std::fs::rename(&output_temp, &options.output)?;
    drop(output_file);

    Ok(())
}

fn stringify_bool(b: bool) -> &'static str {
    match b {
        true => "True",
        false => "False",
    }
}

fn write_indent(string: &mut String, indent: u16) {
    for _ in 0..indent {
        string.push('\t');
    }
}

/// A command code
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
struct CommandCode(u32);

impl CommandCode {
    /// This is likely related to move routes somehow,
    /// Like how the TEXT_DATA command extends the SHOW_TEXT command.
    /// However, I can't find the implementation of this instruction.
    const UNKNOWN_505: Self = Self(505);

    const NOP: Self = Self(0);

    const SHOW_TEXT: Self = Self(101);

    const CONDITONAL_BRANCH: Self = Self(111);

    const CONTROL_SWITCHES: Self = Self(121);

    const TRANSFER_PLAYER: Self = Self(201);

    const SET_MOVEMENT_ROUTE: Self = Self(205);

    const CHANGE_TRANSPARENCY: Self = Self(211);
    const SHOW_ANIMATION: Self = Self(212);
    const SHOW_BALLOON_ICON: Self = Self(213);

    const FADEOUT_SCREEN: Self = Self(221);

    const WAIT: Self = Self(230);

    const TEXT_DATA: Self = Self(401);
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
            Self::CONDITONAL_BRANCH => write!(f, "CONDITONAL_BRANCH"),
            Self::CONTROL_SWITCHES => write!(f, "CONTROL_SWITCHES"),
            Self::TRANSFER_PLAYER => write!(f, "TRANSFER_PLAYER"),
            Self::SET_MOVEMENT_ROUTE => write!(f, "SET_MOVEMENT_ROUTE"),
            Self::CHANGE_TRANSPARENCY => write!(f, "CHANGE_TRANSPARENCY"),
            Self::SHOW_ANIMATION => write!(f, "SHOW_ANIMATION"),
            Self::SHOW_BALLOON_ICON => write!(f, "SHOW_BALLOON_ICON"),
            Self::FADEOUT_SCREEN => write!(f, "FADEOUT_SCREEN"),
            Self::WAIT => write!(f, "WAIT"),
            Self::TEXT_DATA => write!(f, "TEXT_DATA"),
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

    /// Get this as a u8.
    pub fn as_u8(self) -> u8 {
        self as u8
    }
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

    /// Get this as a u8.
    pub fn as_u8(self) -> u8 {
        self as u8
    }

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

/// A command
#[derive(Debug)]
enum Command {
    Nop,
    ShowText {
        face_name: String,
        face_index: u32,
        background: u32,
        position_type: u32,
        lines: Vec<String>,
    },
    ConditionalBranch(ConditionalBranchCommand),
    ControlSwitches {
        start_id: u32,
        end_id: u32,
        value: bool,
    },
    ChangeTransparency {
        set_transparent: bool,
    },
    FadeoutScreen,
    Wait {
        duration: u32,
    },
    ConditionalBranchEnd,
    Unknown {
        code: CommandCode,
        parameters: Vec<rpgmv_types::EventCommandParameter>,
    },
}

#[derive(Debug)]
enum ConditionalBranchCommand {
    Variable {
        lhs_id: u32,
        rhs_id: MaybeRef<u32>,
        operation: ConditionalBranchVariableOperation,
    },
}

#[derive(Debug, Copy, Clone, Hash)]
enum MaybeRef<T> {
    Constant(T),
    Ref(u32),
}

fn parse_event_command_list(
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
                    .as_int()
                    .and_then(|n| u32::try_from(*n).ok())
                    .context("`face_index` is not a `u32`")?;
                let background = event_command.parameters[2]
                    .as_int()
                    .and_then(|n| u32::try_from(*n).ok())
                    .context("`background` is not a string")?;
                let position_type = event_command.parameters[3]
                    .as_int()
                    .and_then(|n| u32::try_from(*n).ok())
                    .context("`position_type` is not a string")?;

                Command::ShowText {
                    face_name,
                    face_index,
                    background,
                    position_type,
                    lines: Vec::new(),
                }
            }
            (_, CommandCode::CONDITONAL_BRANCH) => {
                ensure!(!event_command.parameters.is_empty());
                let kind = event_command.parameters[0]
                    .as_int()
                    .and_then(|value| u8::try_from(*value).ok())
                    .context("`kind` is not a `u32`")?;
                let kind = ConditionalBranchKind::from_u8(kind)?;

                let inner = match kind {
                    ConditionalBranchKind::Variable => {
                        ensure!(event_command.parameters.len() == 5);

                        let lhs_id = event_command.parameters[1]
                            .as_int()
                            .and_then(|value| u32::try_from(*value).ok())
                            .context("`lhs_id` is not a `u32`")?;
                        let is_constant = event_command.parameters[2]
                            .as_int()
                            .and_then(|value| u8::try_from(*value).ok())
                            .context("`is_constant` is not a `u32`")?;
                        let is_constant = is_constant == 0;
                        let rhs_id = event_command.parameters[3]
                            .as_int()
                            .and_then(|value| u32::try_from(*value).ok())
                            .context("`rhs_id` is not a `u32`")?;
                        let rhs_id = if is_constant {
                            MaybeRef::Constant(rhs_id)
                        } else {
                            MaybeRef::Ref(rhs_id)
                        };
                        let operation = event_command.parameters[4]
                            .as_int()
                            .and_then(|value| u8::try_from(*value).ok())
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
            (_, CommandCode::CONTROL_SWITCHES) => {
                ensure!(event_command.parameters.len() == 3);

                let start_id = event_command.parameters[0]
                    .as_int()
                    .and_then(|value| u32::try_from(*value).ok())
                    .context("`start_switch_id` is not a `u32`")?;
                let end_id = event_command.parameters[1]
                    .as_int()
                    .and_then(|value| u32::try_from(*value).ok())
                    .context("`end_switch_id` is not a `u32`")?;
                let value = event_command.parameters[2]
                    .as_int()
                    .and_then(|value| u32::try_from(*value).ok())
                    .context("`value` is not a `u32`")?;
                ensure!(value <= 1);
                let value = value == 0;

                Command::ControlSwitches {
                    start_id,
                    end_id,
                    value,
                }
            }
            (_, CommandCode::CHANGE_TRANSPARENCY) => {
                ensure!(event_command.parameters.len() == 1);
                let value = *event_command.parameters[0]
                    .as_int()
                    .context("parameter is not an int")?;
                ensure!(value > 0 && value <= 1);

                let set_transparent = value == 0;
                Command::ChangeTransparency { set_transparent }
            }
            (_, CommandCode::FADEOUT_SCREEN) => {
                ensure!(event_command.parameters.is_empty());
                Command::FadeoutScreen
            }
            (_, CommandCode::WAIT) => {
                ensure!(event_command.parameters.len() == 1);
                let duration = event_command.parameters[0]
                    .as_int()
                    .and_then(|duration| u32::try_from(*duration).ok())
                    .context("`duration` is not a `u32`")?;

                Command::Wait { duration }
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
