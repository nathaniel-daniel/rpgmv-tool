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
        long = "output",
        short = 'o',
        description = "the path to the output file",
        default = "PathBuf::from(\"out.py\")"
    )]
    output: PathBuf,
}

pub fn exec(options: Options) -> anyhow::Result<()> {
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
        for _ in 0..indent {
            python.push('\t');
        }

        match command {
            Command::ChangeTransparency { set_transparent } => {
                writeln!(&mut python, "ChangeTransparency(set_transparent={})", stringify_bool(set_transparent))?
            }
            Command::Wait { duration } => {
                writeln!(&mut python, "Wait(duration={duration})")?
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

fn stringify_bool(b: bool)  -> &'static str{
    match b {
        true => "True",
        false => "False",
    }
}

/// A command code
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
struct CommandCode(u32);

impl CommandCode {
    const CHANGE_TRANSPARENCY: Self = CommandCode(211);
    
    const WAIT: Self = CommandCode(230);
}

impl std::fmt::Debug for CommandCode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Self::CHANGE_TRANSPARENCY => write!(f, "CHANGE_TRANSPARENCY"),
            Self::WAIT => write!(f, "WAIT"),
            _ => write!(f, "Unknown({})", self.0),
        }
    }
}

/// A command
#[derive(Debug)]
enum Command {
    ChangeTransparency {
        set_transparent: bool,
    },
    Wait {duration: u32},
    Unknown {
        code: CommandCode,
        parameters: Vec<rpgmv_types::EventCommandParameter>,
    },
}

fn parse_event_command_list(
    list: &[rpgmv_types::EventCommand],
) -> anyhow::Result<Vec<(u16, Command)>> {
    let mut ret = Vec::with_capacity(list.len());
    for event_command in list {
        let command_code = CommandCode(event_command.code);

        let command = match command_code {
            CommandCode::CHANGE_TRANSPARENCY => {
                ensure!(event_command.parameters.len() == 1);
                let value = *event_command.parameters[0]
                    .as_int()
                    .context("parameter is not an int")?;
                ensure!(value <= 1);

                let set_transparent = value == 0;
                Command::ChangeTransparency { set_transparent }
            }
            CommandCode::WAIT => {
                ensure!(event_command.parameters.len() == 1);
                let duration = *event_command.parameters[0]
                    .as_int()
                    .context("parameter is not an int")?;
                let duration = u32::try_from(duration)?;
                
                Command::Wait { duration}
            }
            _ => Command::Unknown {
                code: command_code,
                parameters: event_command.parameters.clone(),
            },
        };

        ret.push((event_command.indent, command));
    }
    Ok(ret)
}
