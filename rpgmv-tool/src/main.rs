mod command;

#[derive(Debug, argh::FromArgs)]
#[argh(description = "a CLI tool with utilities to make interacting with rpgmv games easier")]
struct Options {
    #[argh(subcommand)]
    subcommand: SubCommand,
}

#[derive(Debug, argh::FromArgs)]
#[argh(subcommand)]
enum SubCommand {
    Decrypt(self::command::decrypt::Options),
    Commands2Py(self::command::commands2py::Options),
    EncryptPng(self::command::encrypt_png::Options),
}

fn main() -> anyhow::Result<()> {
    let options: Options = argh::from_env();

    match options.subcommand {
        SubCommand::Decrypt(options) => self::command::decrypt::exec(options)?,
        SubCommand::Commands2Py(options) => self::command::commands2py::exec(options)?,
        SubCommand::EncryptPng(options) => self::command::encrypt_png::exec(options)?,
    }

    Ok(())
}
