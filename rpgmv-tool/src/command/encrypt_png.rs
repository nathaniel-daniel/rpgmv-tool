use anyhow::Context;
use std::fs::File;
use std::io::BufReader;
use std::io::BufWriter;
use std::path::PathBuf;
use std::io::Write;

#[derive(Debug, argh::FromArgs)]
#[argh(subcommand, name = "encrypt-png", description = "encrypt a png")]
pub struct Options {
    #[argh(option, long = "input", short = 'i', description = "a png to encrypt")]
    pub input: PathBuf,

    #[argh(option, long = "output", short = 'o', description = "the output file")]
    pub output: PathBuf,

    #[argh(option, long = "key", short = 'k', description = "the key, as hex")]
    pub key: String,
}

pub fn exec(options: Options) -> anyhow::Result<()> {
    let input = File::open(&options.input).with_context(|| {
        format!(
            "failed to open \"{}\" for reading",
            &options.input.display()
        )
    })?;
    let mut input = BufReader::new(input);

    let output = File::create(&options.output).with_context(|| {
        format!(
            "failed to open \"{}\" for writing",
            &options.output.display()
        )
    })?;
    let mut output = BufWriter::new(output);

    let mut key = [0; 16];
    base16ct::mixed::decode(&options.key, &mut key)
        .with_context(|| format!("failed to decode hex \"{}\"", options.key))?;

    let mut writer = rpgmvp::Writer::new(&mut output, key);
    writer.write_header()?;
    std::io::copy(&mut input, &mut writer)?;

    output.flush().context("failed to flush")?;
    let output = output.into_inner()?;
    output.sync_all()?;

    Ok(())
}
