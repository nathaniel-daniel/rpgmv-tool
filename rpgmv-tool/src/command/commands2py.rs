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
    id: Option<u32>,

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
        description = "the path to the output file"
    )]
    output: Option<PathBuf>,

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

    let input_file_kind = FileKind::new(&options.input, true)
        .map(|kind| kind.context("unknown file type"))
        .and_then(std::convert::identity)
        .with_context(|| {
            format!(
                "failed to determine file kind for \"{}\"",
                options.input.display()
            )
        })?;

    if input_file_kind.is_dir() {
        let output = options.output.as_deref().unwrap_or("out".as_ref());
        ensure!(
            options.id.is_none(),
            "the --id flag is unsupported for directories"
        );
        ensure!(
            options.event_page.is_none(),
            "the --event-page flag is unsupported for directories"
        );

        dump_dir(
            &options.input,
            options.dry_run,
            options.overwrite,
            &config,
            output,
        )?;
    } else {
        let id = options
            .id
            .context("the item id must be specified with the --id option")?;
        let output = options.output.as_deref().unwrap_or("out.py".as_ref());

        dump_file(
            input_file_kind,
            &config,
            DumpFileOptions {
                input: &options.input,

                id,
                event_page: options.event_page,

                output,
                dry_run: options.dry_run,
                overwrite: options.overwrite,
            },
        )?;
    }

    Ok(())
}

fn dump_dir(
    input: &Path,
    dry_run: bool,
    overwrite: bool,
    config: &Config,
    output: &Path,
) -> anyhow::Result<()> {
    ensure!(
        overwrite || !output.try_exists()?,
        "output path \"{}\" already exists. Use the --overwrite flag to overwrite",
        output.display()
    );
    if !dry_run {
        try_create_dir(output).context("failed to create output dir")?;
    }

    for dir_entry in std::fs::read_dir(input)? {
        let dir_entry = dir_entry?;
        let file_type = dir_entry.file_type()?;

        if file_type.is_dir() {
            continue;
        }

        let input = dir_entry.path();
        let input_file_kind = FileKind::new(&input, false).with_context(|| {
            format!("failed to determine file kind for \"{}\"", input.display())
        })?;
        let input_str = std::fs::read_to_string(&input)
            .with_context(|| format!("failed to read \"{}\"", input.display()))?;

        let mut output = output.to_path_buf();
        let input_file_kind = match input_file_kind {
            Some(input_file_kind) => input_file_kind,
            None => continue,
        };
        match input_file_kind {
            FileKind::Map => {
                output.push("maps");

                let file_stem = input
                    .file_stem()
                    .context("missing file stem")?
                    .to_str()
                    .context("map name is not valid unicode")?;
                let map_id = extract_map_id(file_stem)?.context("missing map id")?;

                output.push(format!("{map_id:03}"));

                let map: rpgmv_types::Map = serde_json::from_str(&input_str)
                    .with_context(|| format!("failed to parse \"{}\"", input.display()))?;

                for (event_id, event) in map.events.iter().enumerate() {
                    let event = match event {
                        Some(event) => event,
                        None => continue,
                    };
                    let event_id_u32 = u32::try_from(event_id)?;

                    for (page_index, page) in event.pages.iter().enumerate() {
                        if page.list.iter().all(|command| command.code == 0) {
                            continue;
                        }
                        let page_index_u16 = u16::try_from(page_index)?;

                        let file_name =
                            format!("event_{event_id_u32:02}_page_{page_index_u16:02}.py");
                        let output = output.join(file_name);

                        if !dry_run {
                            if let Some(parent) = output.parent() {
                                std::fs::create_dir_all(parent).with_context(|| {
                                    format!("failed to create dir at\"{}\"", parent.display())
                                })?;
                            }
                        }

                        dump_file(
                            input_file_kind,
                            config,
                            DumpFileOptions {
                                input: &input,

                                id: event_id_u32,
                                event_page: Some(page_index_u16),

                                output: &output,
                                dry_run,
                                overwrite,
                            },
                        )?;
                    }
                }
            }
            FileKind::CommonEvents => {
                output.push("common-events");

                let common_events: Vec<Option<rpgmv_types::CommonEvent>> =
                    serde_json::from_str(&input_str)
                        .with_context(|| format!("failed to parse \"{}\"", input.display()))?;

                for (common_event_id, common_event) in common_events.iter().enumerate() {
                    let common_event = match common_event {
                        Some(common_event) => common_event,
                        None => {
                            continue;
                        }
                    };
                    let common_event_id_u32 = u32::try_from(common_event_id)?;

                    let event_name = config
                        .common_events
                        .get(&common_event_id_u32)
                        .unwrap_or(&common_event.name);
                    let output_file_name = format!("{common_event_id_u32:02}_{event_name}.py");
                    let output = output.join(output_file_name);

                    if !dry_run {
                        if let Some(parent) = output.parent() {
                            std::fs::create_dir_all(parent).with_context(|| {
                                format!("failed to create dir at\"{}\"", parent.display())
                            })?;
                        }
                    }

                    dump_file(
                        input_file_kind,
                        config,
                        DumpFileOptions {
                            input: &input,

                            id: common_event_id_u32,
                            event_page: None,

                            output: &output,
                            dry_run,
                            overwrite,
                        },
                    )?;
                }
            }
            FileKind::Troops => {
                output.push("troops");

                let troops: Vec<Option<rpgmv_types::Troop>> = serde_json::from_str(&input_str)
                    .with_context(|| format!("failed to parse \"{}\"", input.display()))?;

                for (troop_id, troop) in troops.iter().enumerate() {
                    let troop = match troop {
                        Some(troop) => troop,
                        None => {
                            continue;
                        }
                    };
                    let troop_id_u32 = u32::try_from(troop_id)?;
                    let troop_name = troop.name.replace('*', "＊");

                    for (page_index, page) in troop.pages.iter().enumerate() {
                        let page_index_u16 = u16::try_from(page_index)?;

                        if page.list.iter().all(|command| command.code == 0) {
                            continue;
                        }

                        let output_file_name =
                            format!("{troop_id_u32:02}_page_{page_index:02}_{troop_name}.py");
                        let output = output.join(output_file_name);

                        if !dry_run {
                            if let Some(parent) = output.parent() {
                                std::fs::create_dir_all(parent).with_context(|| {
                                    format!("failed to create dir at\"{}\"", parent.display())
                                })?;
                            }
                        }

                        dump_file(
                            input_file_kind,
                            config,
                            DumpFileOptions {
                                input: &input,

                                id: troop_id_u32,
                                event_page: Some(page_index_u16),

                                output: &output,
                                dry_run,
                                overwrite,
                            },
                        )?;
                    }
                }
            }
            FileKind::Dir => {
                bail!("input is a dir");
            }
        }
    }

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
    pub fn new(path: &Path, allow_dir: bool) -> anyhow::Result<Option<Self>> {
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

            if extract_map_id(file_stem)?.is_some() {
                return Ok(Some(Self::Map));
            }

            match file_stem {
                "CommonEvents" => return Ok(Some(Self::CommonEvents)),
                "Troops" => return Ok(Some(Self::Troops)),
                _ => {}
            }
        } else if allow_dir {
            return Ok(Some(Self::Dir));
        }

        Ok(None)
    }

    /// Returns `true` if this is a dir.
    pub fn is_dir(self) -> bool {
        matches!(self, Self::Dir)
    }
}

/// Extracts the map number from a file name.
///
/// # Returns
/// Returns `None` if this is not a map.
fn extract_map_id(file_stem: &str) -> anyhow::Result<Option<u16>> {
    let n = match file_stem.strip_prefix("Map") {
        Some(n) => n,
        None => return Ok(None),
    };

    if !n.chars().all(|c| c.is_ascii_digit()) {
        return Ok(None);
    }

    let n: u16 = n.parse().context("failed to parse map number")?;

    Ok(Some(n))
}

/// Try to create a dir.
///
/// Returns false if the dir already exists.
fn try_create_dir<P>(path: P) -> std::io::Result<bool>
where
    P: AsRef<Path>,
{
    match std::fs::create_dir(path) {
        Ok(()) => Ok(true),
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => Ok(false),
        Err(error) => Err(error),
    }
}
