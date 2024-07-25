mod command;
mod config;

use self::command::parse_event_command_list;
use self::command::Command;
use self::command::ConditionalBranchCommand;
use self::command::ControlVariablesValue;
use self::command::MaybeRef;
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
            Command::ShowChoices {
                choices,
                cancel_type,
                default_type,
                position_type,
                background,
            } => {
                write_indent(&mut python, indent);
                writeln!(&mut python, "ShowChoices(")?;

                write_indent(&mut python, indent + 1);
                writeln!(&mut python, "choices=[")?;

                for choice in choices {
                    let choice = escape_string(&choice);

                    write_indent(&mut python, indent + 2);
                    writeln!(&mut python, "'{choice}',")?;
                }

                write_indent(&mut python, indent + 1);
                writeln!(&mut python, "],")?;

                write_indent(&mut python, indent + 1);
                writeln!(&mut python, "cancel_type={cancel_type},")?;

                write_indent(&mut python, indent + 1);
                writeln!(&mut python, "default_type={default_type},")?;

                write_indent(&mut python, indent + 1);
                writeln!(&mut python, "position_type={position_type},")?;

                write_indent(&mut python, indent + 1);
                writeln!(&mut python, "background={background},")?;

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
                let set_transparent = stringify_bool(set_transparent);

                write_indent(&mut python, indent);
                writeln!(
                    &mut python,
                    "ChangeTransparency(set_transparent={set_transparent})"
                )?
            }
            Command::ShowBalloonIcon {
                character_id,
                balloon_id,
                wait,
            } => {
                let wait = stringify_bool(wait);

                write_indent(&mut python, indent);
                writeln!(&mut python, "ShowBalloonIcon(character_id={character_id}, balloon_id={balloon_id}, wait={wait})")?
            }
            Command::FadeoutScreen => {
                write_indent(&mut python, indent);
                writeln!(&mut python, "FadeoutScreen()")?
            }
            Command::FadeinScreen => {
                write_indent(&mut python, indent);
                writeln!(&mut python, "FadeinScreen()")?
            }
            Command::FlashScreen {
                color,
                duration,
                wait,
            } => {
                let wait = stringify_bool(wait);

                write_indent(&mut python, indent);
                writeln!(
                    &mut python,
                    "FlashScreen(color={color:?}, duration={duration}, wait={wait})"
                )?
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
            Command::WhenEnd => {
                // Trust indents over end commands
            }
            Command::Else => {
                write_indent(&mut python, indent);
                writeln!(&mut python, "else:")?;
            }
            Command::ConditionalBranchEnd => {
                // Trust indents over end commands
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
