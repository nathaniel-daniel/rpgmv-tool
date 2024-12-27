use anyhow::Context;
use flate2::read::ZlibDecoder;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::path::PathBuf;

#[derive(Debug, argh::FromArgs)]
#[argh(
    subcommand,
    name = "unpack-save",
    description = "a tool to unpack a save"
)]
pub struct Options {
    #[argh(positional)]
    pub input: PathBuf,

    #[argh(positional)]
    pub output: PathBuf,

    #[argh(switch, long = "pretty", description = "whether to format the output")]
    pub pretty: bool,
}

fn prepare_decompress_input(input: &str) -> anyhow::Result<Vec<u8>> {
    let mut output = Vec::new();
    for value in input.encode_utf16() {
        let value = u8::try_from(value)?;
        output.push(value);
    }
    Ok(output)
}

pub fn exec(options: Options) -> anyhow::Result<()> {
    let input = std::fs::read_to_string(&options.input)
        .with_context(|| format!("failed to read file at \"{}\"", options.input.display()))?;
    let input = prepare_decompress_input(&input).context("failed to prepare decompress input")?;

    let mut decoder = ZlibDecoder::new(std::io::Cursor::new(input));

    let mut output_string = String::new();
    decoder.read_to_string(&mut output_string)?;

    if options.pretty {
        let parsed: serde_json::Value = serde_json::from_str(&output_string)?;
        output_string = serde_json::to_string_pretty(&parsed)?;
    }

    let tmp_out_path = nd_util::with_push_extension(&options.output, "tmp");
    let mut out_file = File::create(&tmp_out_path)
        .with_context(|| format!("failed to create file at \"{}\"", tmp_out_path.display()))?;
    out_file.write_all(output_string.as_bytes())?;
    out_file.flush()?;
    out_file.sync_all()?;

    std::fs::rename(&tmp_out_path, &options.output).context("failed to rename output file")?;

    Ok(())
}
