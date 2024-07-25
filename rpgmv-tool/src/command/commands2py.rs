mod config;

use self::config::Config;
use anyhow::bail;
use anyhow::ensure;
use anyhow::Context;
use std::fmt::Write as _;
use std::fs::File;
use std::io::Write;
use std::path::Path;
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

    let input_file_kind = FileKind::new(&options.input).with_context(|| {
        format!(
            "failed to determine file kind for \"{}\"",
            options.input.display()
        )
    })?;
    let input_str = std::fs::read_to_string(&options.input)
        .with_context(|| format!("failed to read \"{}\"", options.input.display()))?;
    let event_commands = match input_file_kind {
        FileKind::Map => {
            let mut map: rpgmv_types::Map = serde_json::from_str(&input_str)
                .with_context(|| format!("failed to parse \"{}\"", options.input.display()))?;

            let mut event = usize::try_from(options.event_id)
                .ok()
                .and_then(|event_id| {
                    if event_id >= map.events.len() {
                        return None;
                    }

                    map.events.swap_remove(event_id)
                })
                .with_context(|| format!("no event with id {}", options.event_id))?;
            ensure!(event.id == options.event_id);

            let event_page_index = match options.event_page {
                Some(event_page) => event_page,
                None if event.pages.len() == 1 => 0,
                None => {
                    bail!(
                        "found multiple event pages. specify which one with the --event-page option"
                    )
                }
            };
            let event_page_index = usize::from(event_page_index);
            ensure!(
                event_page_index < event.pages.len(),
                "no event page with index {event_page_index}"
            );
            let event_page = event.pages.swap_remove(event_page_index);

            event_page.list
        }
        FileKind::CommonEvents => {
            let mut common_events: Vec<Option<rpgmv_types::CommonEvent>> =
                serde_json::from_str(&input_str)
                    .with_context(|| format!("failed to parse \"{}\"", options.input.display()))?;

            let event = usize::try_from(options.event_id)
                .ok()
                .and_then(|event_id| {
                    if event_id >= common_events.len() {
                        return None;
                    }

                    common_events.swap_remove(event_id)
                })
                .with_context(|| format!("no event with id {}", options.event_id))?;
            ensure!(event.id == options.event_id);

            ensure!(
                options.event_page.is_none(),
                "common events do not have pages, remove the --event-page option"
            );

            event.list
        }
    };

    let commands = parse_event_command_list(&event_commands)?;

    let mut python = String::new();
    for (indent, command) in commands {
        match command {
            Command::Nop => {}
            Command::ShowText {
                face_name,
                face_index,
                background,
                position_type,
                lines,
            } => {
                write_indent(&mut python, indent);
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
                    let line = escape_string(&line);

                    write_indent(&mut python, indent + 2);
                    writeln!(&mut python, "'{line}',")?;
                }

                write_indent(&mut python, indent + 1);
                writeln!(&mut python, "],")?;

                write_indent(&mut python, indent);
                writeln!(&mut python, ")")?;
            }
            Command::ConditionalBranch(command) => {
                write_indent(&mut python, indent);
                write!(&mut python, "if ")?;
                match command {
                    ConditionalBranchCommand::Switch { id, check_true } => {
                        let name = config.get_switch_name(id);
                        let check_true_str = if check_true { "" } else { "not " };
                        writeln!(&mut python, "{check_true_str}{name}:")?;
                    }
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
            Command::CommonEvent { id } => {
                let name = config.get_common_event_name(id);

                write_indent(&mut python, indent);
                writeln!(&mut python, "{name}()")?;
            }
            Command::ControlSwitches {
                start_id,
                end_id,
                value,
            } => {
                let mut iter = (start_id..(end_id + 1)).peekable();
                let value = stringify_bool(value);

                write_indent(&mut python, indent);
                while let Some(id) = iter.next() {
                    let name = config.get_switch_name(id);

                    writeln!(&mut python, "{name} = {value}")?;
                    if iter.peek().is_some() {
                        write_indent(&mut python, indent);
                    }
                }
            }
            Command::ControlVariables {
                start_variable_id,
                end_variable_id,
                operation,
                value,
            } => {
                let operation = operation.as_str();
                let value = match value {
                    ControlVariablesValue::Constant { value } => value.to_string(),
                    ControlVariablesValue::Variable { id } => config.get_variable_name(id),
                    ControlVariablesValue::Random { start, stop } => {
                        format!("random.randrange(start={start}, stop={stop})")
                    }
                };
                for variable_id in start_variable_id..(end_variable_id + 1) {
                    let name = config.get_variable_name(variable_id);
                    write_indent(&mut python, indent);
                    writeln!(&mut python, "{name} {operation} {value}")?;
                }
            }
            Command::ChangeTransparency { set_transparent } => {
                write_indent(&mut python, indent);
                writeln!(
                    &mut python,
                    "ChangeTransparency(set_transparent={})",
                    stringify_bool(set_transparent)
                )?
            }
            Command::FadeoutScreen => {
                write_indent(&mut python, indent);
                writeln!(&mut python, "FadeoutScreen()")?
            }
            Command::FadeinScreen => {
                write_indent(&mut python, indent);
                writeln!(&mut python, "FadeinScreen()")?
            }
            Command::Wait { duration } => {
                write_indent(&mut python, indent);
                writeln!(&mut python, "Wait(duration={duration})")?
            }
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
            } => {
                let picture_name = escape_string(&picture_name);
                let x = match x {
                    MaybeRef::Constant(value) => value.to_string(),
                    MaybeRef::Ref(id) => config.get_variable_name(id),
                };
                let y = match y {
                    MaybeRef::Constant(value) => value.to_string(),
                    MaybeRef::Ref(id) => config.get_variable_name(id),
                };

                write_indent(&mut python, indent);
                writeln!(&mut python, "ShowPicture(")?;

                write_indent(&mut python, indent + 1);
                writeln!(&mut python, "picture_id={picture_id},")?;

                write_indent(&mut python, indent + 1);
                writeln!(&mut python, "picture_name='{picture_name}',")?;

                write_indent(&mut python, indent + 1);
                writeln!(&mut python, "origin={origin},")?;

                write_indent(&mut python, indent + 1);
                writeln!(&mut python, "x={x},")?;

                write_indent(&mut python, indent + 1);
                writeln!(&mut python, "y={y},")?;

                write_indent(&mut python, indent + 1);
                writeln!(&mut python, "scale_x={scale_x},")?;

                write_indent(&mut python, indent + 1);
                writeln!(&mut python, "scale_y={scale_y},")?;

                write_indent(&mut python, indent + 1);
                writeln!(&mut python, "opacity={opacity},")?;

                write_indent(&mut python, indent + 1);
                writeln!(&mut python, "blend_mode={blend_mode},")?;

                write_indent(&mut python, indent);
                writeln!(&mut python, ")")?;
            }
            Command::ErasePicture { picture_id } => {
                write_indent(&mut python, indent);
                writeln!(&mut python, "ErasePicture(picture_id={picture_id})")?;
            }
            Command::PlaySe { audio } => {
                let audio_name = escape_string(&audio.name);

                write_indent(&mut python, indent);
                writeln!(&mut python, "PlaySe(")?;

                write_indent(&mut python, indent + 1);
                writeln!(&mut python, "audio=AudioFile(")?;

                write_indent(&mut python, indent + 2);
                writeln!(&mut python, "name='{audio_name}',")?;

                write_indent(&mut python, indent + 2);
                writeln!(&mut python, "pan={},", audio.pan)?;

                write_indent(&mut python, indent + 2);
                writeln!(&mut python, "pitch={},", audio.pitch)?;

                write_indent(&mut python, indent + 2);
                writeln!(&mut python, "volume={},", audio.volume)?;

                write_indent(&mut python, indent + 1);
                writeln!(&mut python, "),")?;

                write_indent(&mut python, indent);
                writeln!(&mut python, ")")?;
            }
            Command::ChangeSkill {
                actor_id,
                is_learn_skill,
                skill_id,
            } => {
                let actor_arg = match actor_id {
                    MaybeRef::Constant(actor_id) => {
                        let name = config.get_actor_name(actor_id);
                        format!("actor={name}")
                    }
                    MaybeRef::Ref(variable_id) => {
                        let name = config.get_variable_name(variable_id);
                        format!("actor_id={name}")
                    }
                };
                let fn_name = if is_learn_skill {
                    "LearnSkill"
                } else {
                    "ForgetSkill"
                };
                let skill = config.get_skill_name(skill_id);

                write_indent(&mut python, indent);
                writeln!(&mut python, "{fn_name}({actor_arg}, skill={skill})")?;
            }
            Command::Else => {
                write_indent(&mut python, indent);
                writeln!(&mut python, "else:")?;
            }
            Command::ConditionalBranchEnd => {
                // Trust indents over branch ends
            }
            Command::Unknown { code, parameters } => {
                write_indent(&mut python, indent);
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

#[derive(Debug, Clone, Copy)]
enum FileKind {
    Map,
    CommonEvents,
}

impl FileKind {
    /// Try to extract a file kind from a path.
    pub fn new(path: &Path) -> anyhow::Result<Self> {
        let file_name = path
            .file_name()
            .context("missing file name")?
            .to_str()
            .context("file name is not unicode")?;
        let (file_stem, extension) = file_name
            .rsplit_once('.')
            .context("file name has no extension")?;
        ensure!(extension == "json", "file must be json");

        if let Some(n) = file_stem.strip_prefix("Map") {
            if n.chars().all(|c| c.is_ascii_alphanumeric()) {
                return Ok(Self::Map);
            }
        }

        if file_stem == "CommonEvents" {
            return Ok(Self::CommonEvents);
        }

        bail!("unknown file type")
    }
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

fn escape_string(input: &str) -> String {
    input.replace('\'', "\\'")
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

    const COMMON_EVENT: Self = Self(117);

    const CONTROL_SWITCHES: Self = Self(121);
    const CONTROL_VARIABLES: Self = Self(122);

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

    const ERASE_PICTURE: Self = Self(235);

    const PLAY_SE: Self = Self(250);

    const CHANGE_SKILL: Self = Self(318);

    const TEXT_DATA: Self = Self(401);

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
            Self::CONDITONAL_BRANCH => write!(f, "CONDITONAL_BRANCH"),
            Self::COMMON_EVENT => write!(f, "COMMON_EVENT"),
            Self::CONTROL_SWITCHES => write!(f, "CONTROL_SWITCHES"),
            Self::CONTROL_VARIABLES => write!(f, "CONTROL_VARIABLES"),
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
            Self::ERASE_PICTURE => write!(f, "ERASE_PICTURE"),
            Self::PLAY_SE => write!(f, "PLAY_SE"),
            Self::CHANGE_SKILL => write!(f, "CHANGE_SKILL"),
            Self::TEXT_DATA => write!(f, "TEXT_DATA"),
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
    ChangeTransparency {
        set_transparent: bool,
    },
    FadeoutScreen,
    FadeinScreen,
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
    Else,
    ConditionalBranchEnd,
    Unknown {
        code: CommandCode,
        parameters: Vec<serde_json::Value>,
    },
}

#[derive(Debug)]
enum ConditionalBranchCommand {
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
enum ControlVariablesValue {
    Constant { value: u32 },
    Variable { id: u32 },
    Random { start: u32, stop: u32 },
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
            (_, CommandCode::CHANGE_TRANSPARENCY) => {
                ensure!(event_command.parameters.len() == 1);
                let value = event_command.parameters[0]
                    .as_i64()
                    .context("parameter is not an int")?;
                ensure!(value > 0 && value <= 1);

                let set_transparent = value == 0;
                Command::ChangeTransparency { set_transparent }
            }
            (_, CommandCode::FADEOUT_SCREEN) => {
                ensure!(event_command.parameters.is_empty());
                Command::FadeoutScreen
            }
            (_, CommandCode::FADEIN_SCREEN) => {
                ensure!(event_command.parameters.is_empty());
                Command::FadeinScreen
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
