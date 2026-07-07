use anyhow::Context;
use clap::Parser;
use std::fs::File;
use std::io::BufReader;
use std::io::BufWriter;
use std::io::Write;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(about = "Encrypt a png")]
pub struct Options {
    #[arg(long = "input", short = 'i', help = "A png to encrypt")]
    pub input: PathBuf,

    #[arg(long = "output", short = 'o', help = "The output file")]
    pub output: PathBuf,

    #[arg(long = "key", short = 'k', help = "The key, as hex")]
    pub key: String,
}

pub fn exec(options: Options) -> anyhow::Result<()> {
    let input = File::open(&options.input)
        .with_context(|| format!("failed to open \"{}\" for reading", options.input.display()))?;
    let mut input = BufReader::new(input);

    let output = File::create(&options.output).with_context(|| {
        format!(
            "failed to open \"{}\" for writing",
            options.output.display()
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
