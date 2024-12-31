use anyhow::Context;
use flate2::bufread::ZlibEncoder;
use flate2::Compression;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::path::PathBuf;

#[derive(Debug, argh::FromArgs)]
#[argh(subcommand, name = "pack-save", description = "a tool to pack a save")]
pub struct Options {
    #[argh(positional)]
    pub input: PathBuf,

    #[argh(positional)]
    pub output: PathBuf,
}

fn prepare_compress_output(input: &[u8]) -> String {
    let mut output = String::new();
    for value in input.iter().copied() {
        output.push(char::from(value));
    }
    output
}

pub fn exec(options: Options) -> anyhow::Result<()> {
    let input = std::fs::read_to_string(&options.input)
        .with_context(|| format!("failed to read file at \"{}\"", options.input.display()))?;
    let input: serde_json::Value =
        serde_json::from_str(&input).context("save file is not valid json")?;
    let input = serde_json::to_string(&input)?;

    let mut encoder = ZlibEncoder::new(std::io::Cursor::new(input), Compression::fast());
    let mut output = Vec::new();
    encoder.read_to_end(&mut output)?;

    let output = prepare_compress_output(&output);

    let tmp_out_path = nd_util::with_push_extension(&options.output, "tmp");
    let mut out_file = File::create(&tmp_out_path)
        .with_context(|| format!("failed to create file at \"{}\"", tmp_out_path.display()))?;
    out_file.write_all(output.as_bytes())?;
    out_file.flush()?;
    out_file.sync_all()?;

    std::fs::rename(&tmp_out_path, &options.output).context("failed to rename output file")?;

    Ok(())
}
