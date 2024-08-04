mod command;
mod config;
mod generate;

use self::command::parse_event_command_list;
use self::command::Command;
use self::command::ConditionalBranchCommand;
use self::command::ControlVariablesValue;
use self::command::ControlVariablesValueGameData;
use self::command::MaybeRef;
use self::config::Config;
use self::generate::commands2py;
use anyhow::bail;
use anyhow::ensure;
use anyhow::Context;
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

    #[argh(option, long = "id", description = "id of the item to convert")]
    id: u32,

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
        switch,
        long = "dry-run",
        description = "avoid writing the output files"
    )]
    dry_run: bool,

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

    let input_file_kind = FileKind::new(&options.input, true).with_context(|| {
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

            let mut event = usize::try_from(options.id)
                .ok()
                .and_then(|id| {
                    if id >= map.events.len() {
                        return None;
                    }

                    map.events.swap_remove(id)
                })
                .with_context(|| format!("no event with id {}", options.id))?;
            ensure!(event.id == options.id);

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

            let event = usize::try_from(options.id)
                .ok()
                .and_then(|event_id| {
                    if event_id >= common_events.len() {
                        return None;
                    }

                    common_events.swap_remove(event_id)
                })
                .with_context(|| format!("no event with id {}", options.id))?;
            ensure!(event.id == options.id);

            ensure!(
                options.event_page.is_none(),
                "common events do not have pages, remove the --event-page option"
            );

            event.list
        }
        FileKind::Troops => {
            let mut troops: Vec<Option<rpgmv_types::Troop>> = serde_json::from_str(&input_str)
                .with_context(|| format!("failed to parse \"{}\"", options.input.display()))?;

            let mut troop = usize::try_from(options.id)
                .ok()
                .and_then(|event_id| {
                    if event_id >= troops.len() {
                        return None;
                    }

                    troops.swap_remove(event_id)
                })
                .with_context(|| format!("no troop with id {}", options.id))?;

            let event_page_index = match options.event_page {
                Some(event_page) => event_page,
                None if troop.pages.len() == 1 => 0,
                None => {
                    bail!(
                        "found multiple event pages. specify which one with the --event-page option"
                    )
                }
            };
            let event_page_index = usize::from(event_page_index);
            ensure!(
                event_page_index < troop.pages.len(),
                "no event page with index {event_page_index}"
            );
            let event_page = troop.pages.swap_remove(event_page_index);

            event_page.list
        }
        FileKind::Dir => {
            bail!("input is a dir. This is currently unsupported.");
        }
    };

    let commands =
        parse_event_command_list(&event_commands).context("failed to parse event command list")?;
    let mut python = String::with_capacity(event_commands.len() * 16);
    commands2py(&config, &commands, &mut python)?;

    if !options.dry_run {
        write_string_safe(&options.output, &python)?;
    }

    Ok(())
}

fn write_string_safe<P>(path: P, s: &str) -> anyhow::Result<()>
where
    P: AsRef<Path>,
{
    let path = path.as_ref();
    let path_temp = nd_util::with_push_extension(path, "tmp");
    let mut file = File::create(&path_temp)
        .with_context(|| format!("failed to open \"{}\"", path_temp.display()))?;
    file.write_all(s.as_bytes())?;
    file.flush()?;
    file.sync_all()?;
    std::fs::rename(&path_temp, path)?;
    drop(file);

    Ok(())
}

#[derive(Debug, Clone, Copy)]
enum FileKind {
    Map,
    CommonEvents,
    Troops,
    Dir,
}

impl FileKind {
    /// Try to extract a file kind from a path.
    pub fn new(path: &Path, allow_dir: bool) -> anyhow::Result<Self> {
        let metadata = match path.metadata() {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                bail!("path \"{}\" does not exist", path.display());
            }
            Err(error) => {
                return Err(error)
                    .with_context(|| format!("failed to get metadata for \"{}\"", path.display()));
            }
        };
        let is_file = !metadata.is_dir();

        if is_file {
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

            match file_stem {
                "CommonEvents" => return Ok(Self::CommonEvents),
                "Troops" => return Ok(Self::Troops),
                _ => {}
            }
        } else if allow_dir {
            return Ok(Self::Dir);
        }

        bail!("unknown file type")
    }
}
