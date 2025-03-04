mod command;

#[derive(Debug, argh::FromArgs)]
#[argh(description = "a CLI tool with utilities to make interacting with rpgmz games easier")]
struct Options {
    #[argh(subcommand)]
    subcommand: SubCommand,
}

#[derive(Debug, argh::FromArgs)]
#[argh(subcommand)]
enum SubCommand {
    Commands2Py(self::command::commands2py::Options),
    UnpackSave(self::command::unpack_save::Options),
    PackSave(self::command::pack_save::Options),
    Decrypt(self::command::decrypt::Options),
    EncryptPng(self::command::encrypt_png::Options),
}

fn main() -> anyhow::Result<()> {
    let options: Options = argh::from_env();

    match options.subcommand {
        SubCommand::Commands2Py(options) => self::command::commands2py::exec(options)?,
        SubCommand::UnpackSave(options) => self::command::unpack_save::exec(options)?,
        SubCommand::PackSave(options) => self::command::pack_save::exec(options)?,
        SubCommand::Decrypt(options) => self::command::decrypt::exec(options)?,
        SubCommand::EncryptPng(options) => self::command::encrypt_png::exec(options)?,
    }

    Ok(())
}
