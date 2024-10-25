use anyhow::anyhow;
use anyhow::bail;
use anyhow::ensure;
use anyhow::Context;
use glob::glob;
use std::fs::File;
use std::io::BufReader;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

#[derive(Debug, argh::FromArgs)]
#[argh(subcommand, name = "decrypt", description = "decrypt a file")]
pub struct Options {
    #[argh(option, long = "input", short = 'i', description = "a file to decrypt")]
    pub input: Vec<PathBuf>,

    #[argh(
        option,
        long = "glob-input",
        description = "a glob of input files to decrypt"
    )]
    pub glob_input: Vec<String>,

    #[argh(
        option,
        long = "output",
        short = 'o',
        description = "the output folder"
    )]
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

/// Interface inspired by mv.
/// See: https://man7.org/linux/man-pages/man1/mv.1p.html
pub fn exec(options: Options) -> anyhow::Result<()> {
    let mut inputs = options.input;
    for input in options.glob_input {
        let iter = glob(&input)?;
        for input in iter {
            let input = input?;

            inputs.push(input);
        }
    }

    ensure!(!inputs.is_empty(), "need at least 1 input");

    let output_metadata = try_metadata(&options.output)
        .with_context(|| format!("failed to stat \"{}\"", options.output.display()))?;

    // If the output is a directory, use the vector impl.
    match output_metadata {
        Some(metadata) if metadata.is_dir() => {
            return exec_vector(&inputs, &options.output);
        }
        Some(_) | None => {}
    }

    if inputs.len() == 1 {
        let input = &inputs[0];

        // For file destinations or non-existent destinations
        // We filter out directory outputs earlier.
        exec_scalar(input, &options.output)
    } else {
        // We can't use the scalar impl since there must be more than 1 input.
        // We can't use the vector impl since the target output is either a file or does not exist.
        // Assume the user wanted to use the vector impl for error, since there is more than 1 input.
        Err(anyhow!(
            "\"{}\" is not a directory or does not exist",
            options.output.display()
        ))
    }
}

fn exec_scalar(input: &Path, output: &Path) -> anyhow::Result<()> {
    decrypt_single_file(input, output)
}

fn exec_vector(inputs: &[PathBuf], output: &Path) -> anyhow::Result<()> {
    for input in inputs.iter() {
        let input_file_name = input
            .file_name()
            .with_context(|| format!("failed to get file name from \"{}\"", &input.display()))?;

        let output = {
            let mut path = output.join(input_file_name);
            path.set_extension("png");
            path
        };

        decrypt_single_file(input, &output)?;
    }

    Ok(())
}

fn decrypt_single_file(input: &Path, output: &Path) -> anyhow::Result<()> {
    let output_metadata =
        try_metadata(output).with_context(|| format!("failed to stat \"{}\"", output.display()))?;

    if output_metadata.is_some() {
        bail!(
            "output path \"{}\" exists, refusing to overwrite",
            output.display()
        );
    }

    let file =
        File::open(input).with_context(|| format!("failed to open \"{}\"", input.display()))?;

    let file = BufReader::new(file);
    let mut reader = rpgmvp::Reader::new(file);
    reader.read_header().context("invalid header")?;
    let key = reader.extract_key().context("failed to extract key")?;
    let key_hex = base16ct::lower::encode_string(&key);
    println!("Key for \"{}\": {}", input.display(), key_hex);

    let output_tmp = nd_util::with_push_extension(output, "tmp");
    let mut writer = File::create(&output_tmp)
        .with_context(|| format!("failed to open \"{}\"", output_tmp.display()))?;
    std::io::copy(&mut reader, &mut writer)?;
    writer.flush()?;
    writer.sync_all()?;
    std::fs::rename(&output_tmp, output)?;

    Ok(())
}
