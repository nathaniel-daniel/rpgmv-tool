mod command;
mod util;

use clap::Parser;

const LONG_VERSION: &str = concat!(env!("CARGO_PKG_VERSION"), " ", "(", env!("GIT_REV"), ")");

#[derive(Debug, Parser)]
#[command(
    about = "A CLI tool with utilities to make interacting with RPGMaker MV games easier",
    long_about = None,
    long_version = LONG_VERSION
)]
struct Options {
    #[command(subcommand)]
    subcommand: SubCommand,
}

#[derive(Debug, clap::Subcommand)]
enum SubCommand {
    Decrypt(self::command::decrypt::Options),
    #[command(name = "commands2py")]
    Commands2Py(self::command::commands2py::Options),
    EncryptPng(self::command::encrypt_png::Options),
    GenerateCompletions(self::command::generate_completions::Options),
    CheckLineSize(self::command::check_line_size::Options),
}

fn main() -> anyhow::Result<()> {
    let options = Options::parse();

    match options.subcommand {
        SubCommand::Decrypt(options) => self::command::decrypt::exec(options)?,
        SubCommand::Commands2Py(options) => self::command::commands2py::exec(options)?,
        SubCommand::EncryptPng(options) => self::command::encrypt_png::exec(options)?,
        SubCommand::GenerateCompletions(options) => {
            self::command::generate_completions::exec(options)?
        }
        SubCommand::CheckLineSize(options) => self::command::check_line_size::exec(options)?,
    }

    Ok(())
}
