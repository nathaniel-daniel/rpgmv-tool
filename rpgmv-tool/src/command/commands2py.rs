mod command;
mod config;
mod file_sink;
mod generate;

use self::command::parse_event_command_list;
use self::command::Command;
use self::command::ConditionalBranchCommand;
use self::command::ControlVariablesValue;
use self::command::ControlVariablesValueGameData;
use self::command::MaybeRef;
use self::config::Config;
use self::file_sink::FileSink;
use self::generate::commands2py;
use anyhow::bail;
use anyhow::ensure;
use anyhow::Context;
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

    #[argh(
        switch,
        long = "overwrite",
        description = "whether to overwrite the output, if it exists"
    )]
    overwrite: bool,
}

pub fn exec(options: Options) -> anyhow::Result<()> {
    /*
    let current_exe = std::env::current_exe().context("failed to get current exe")?;
    let current_exe_modified = std::fs::metadata(current_exe)
        .context("failed to get metadata for current exe")?
        .modified();
    */

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
    dump_file(
        input_file_kind,
        &config,
        DumpFileOptions {
            input: &options.input,

            id: options.id,
            event_page: options.event_page,

            output: &options.output,
            dry_run: options.dry_run,
            overwrite: options.overwrite,
        },
    )?;

    Ok(())
}

#[derive(Debug)]
struct DumpFileOptions<'a> {
    input: &'a Path,

    id: u32,
    event_page: Option<u16>,

    output: &'a Path,
    dry_run: bool,
    overwrite: bool,
}

fn dump_file(
    input_file_kind: FileKind,
    config: &Config,
    options: DumpFileOptions<'_>,
) -> anyhow::Result<()> {
    let input_str = std::fs::read_to_string(options.input)
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
            bail!("input is a dir");
        }
    };

    let commands =
        parse_event_command_list(&event_commands).context("failed to parse event command list")?;
    let mut file_sink = FileSink::new(options.output, options.dry_run, options.overwrite)?;

    commands2py(config, &commands, &mut file_sink)?;

    file_sink.finish()?;

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
