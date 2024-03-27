use anyhow::bail;
use anyhow::Context;
use std::fs::File;
use std::io::BufReader;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

#[derive(Debug, argh::FromArgs)]
#[argh(subcommand, name = "decrypt", description = "decrypt a file")]
pub struct Options {
    #[argh(positional, description = "the file to decrypt")]
    pub input: PathBuf,

    #[argh(positional, description = "the output folder")]
    pub output: PathBuf,
}

/// Try to get metadata for a path
fn try_metadata<P>(path: P) -> std::io::Result<Option<std::fs::Metadata>>
where
    P: AsRef<Path>,
{
    match std::fs::metadata(path) {
        Ok(metadata) => Ok(Some(metadata)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error),
    }
}

pub fn exec(options: Options) -> anyhow::Result<()> {
    let output_parent_metadata = try_metadata(&options.output)
        .with_context(|| format!("failed to stat \"{}\"", options.output.display()))?;

    let input_file_name = options.input.file_name().with_context(|| {
        format!(
            "failed to get file name from \"{}\"",
            &options.input.display()
        )
    })?;

    let output = {
        let mut path = options.output.join(input_file_name);
        path.set_extension("png");
        path
    };

    match output_parent_metadata {
        Some(metadata) if !metadata.is_dir() => {
            bail!("output path exists, refusing to overwrite");
        }
        Some(_metadata) => {}
        None => {
            bail!("output path does not exist")
        }
    }

    let output_metadata = try_metadata(&output)
        .with_context(|| format!("failed to stat \"{}\"", output.display()))?;

    if output_metadata.is_some() {
        bail!("output path exists, refusing to overwrite");
    }

    let file = File::open(&options.input)
        .with_context(|| format!("failed to open \"{}\"", options.input.display()))?;
    let file = BufReader::new(file);
    let mut reader = rpgmvp::Reader::new(file);
    reader.read_header().context("invalid header")?;
    reader.extract_key().context("failed to extract key")?;

    let output_tmp = nd_util::with_push_extension(&output, "tmp");
    let mut writer = File::create(&output_tmp)
        .with_context(|| format!("failed to open \"{}\"", output_tmp.display()))?;
    std::io::copy(&mut reader, &mut writer)?;
    writer.flush()?;
    writer.sync_all()?;
    std::fs::rename(&output_tmp, output)?;

    Ok(())
}
