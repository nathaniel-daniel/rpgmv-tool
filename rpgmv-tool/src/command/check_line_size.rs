use clap::Parser;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(about = "Check a game to see if any text lines overflow their boxes")]
pub struct Options {
    #[arg(help = "The path to the game to check", default_value = ".")]
    pub input: PathBuf,
}

pub fn exec(options: Options) -> anyhow::Result<()> {
    for entry in rpgmv_tool_util::check_line_size(&options.input)? {
        let rpgmv_tool_util::CheckLineSizeEntry {
            file,
            line,
            text_width,
            target_width,
            suggested_line,
        } = entry;

        println!("{file}");
        println!("  Text: \"{line}\"");
        println!("  width={text_width} vs target_width={target_width}");
        println!("  suggested_text=\"{suggested_line}\"");
    }

    Ok(())
}
