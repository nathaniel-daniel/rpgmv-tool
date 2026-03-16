mod check_line_size;
pub mod message_parser;
mod util;

pub use self::check_line_size::CheckLineSizeEntry;
pub use self::check_line_size::check_line_size;
use crate::util::try_exists;
use anyhow::ensure;
use std::path::Path;

pub fn is_game_mv<P>(game_path: P) -> anyhow::Result<bool>
where
    P: AsRef<Path>,
{
    let game_path = game_path.as_ref();

    // Sanity check to ensure this is a game.
    {
        let node_dll_path = game_path.join("node.dll");
        let node_dll_path_exists = try_exists(&node_dll_path)?;
        ensure!(node_dll_path_exists);
    }

    let www_path = game_path.join("www");

    // If the www dir exists, this is probably an MV game.
    // MZ games put all their data in the same dir as the exe.
    try_exists(&www_path)
}

/// Try to parse a map name.
///
/// Returns the Map id if the given name was a valid map name.
pub fn parse_map_name(name: &str) -> Option<u16> {
    // Map\d\d\d.json
    name.strip_prefix("Map")?
        .strip_suffix(".json")?
        .parse()
        .ok()
}
