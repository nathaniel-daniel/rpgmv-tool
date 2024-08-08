use super::AudioFile;
use super::Encounter;
use super::Event;

/// A Map
#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Map {
    /// ?
    #[serde(rename = "autoplayBgm")]
    pub autoplay_bgm: bool,

    /// ?
    #[serde(rename = "autoplayBgs")]
    pub autoplay_bgs: bool,

    /// ?
    #[serde(rename = "battleback1Name")]
    pub battleback1_name: String,

    /// ?
    #[serde(rename = "battleback2Name")]
    pub battleback2_name: String,

    /// ?
    pub bgm: AudioFile,

    /// ?
    pub bgs: AudioFile,

    /// ?
    #[serde(rename = "disableDashing")]
    pub disable_dashing: bool,

    /// ?
    #[serde(rename = "displayName")]
    pub display_name: String,

    /// ?
    #[serde(rename = "encounterList")]
    pub encounter_list: Vec<Encounter>,

    /// ?
    #[serde(rename = "encounterStep")]
    pub encounter_step: u32,

    /// ?
    pub height: u32,

    /// ?
    pub note: String,

    /// ?
    #[serde(rename = "parallaxLoopX")]
    pub parallax_loop_x: bool,

    /// ?
    #[serde(rename = "parallaxLoopY")]
    pub parallax_loop_y: bool,

    /// ?
    #[serde(rename = "parallaxName")]
    pub parallax_name: String,

    /// ?
    #[serde(rename = "parallaxShow")]
    pub parallax_show: bool,

    /// ?
    #[serde(rename = "parallaxSx")]
    pub parallax_sx: u32,

    /// ?
    #[serde(rename = "parallaxSy")]
    pub parallax_sy: u32,

    /// ?
    #[serde(rename = "scrollType")]
    pub scroll_type: u32,

    /// ?
    #[serde(rename = "specifyBattleback")]
    pub specify_battleback: bool,

    /// ?
    #[serde(rename = "tilesetId")]
    pub tileset_id: u32,

    /// ?
    pub width: u32,

    /// ?
    pub data: Vec<u32>,

    /// ?
    pub events: Vec<Option<Event>>,
}

#[cfg(test)]
mod test {
    use super::*;
    // Taken from https://github.com/pokemon-essentials/pokemon-essentials/blob/99836e294e588411d44131e4ae878466d5fbbd3a/project/data/Map001.json.
    const MAP_1: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/test-data/maps/Map001.json"
    ));

    // Taken from https://github.com/samuelcardillo/MMORPGMaker-MV/blob/c9dc5f0c4cfe6d1e7d0111f94bc5e7e2ebc16ecd/data/Map002.json.
    const MAP_2: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/test-data/maps/Map002.json"
    ));

    // Taken from https://github.com/craftadria/Timetrollergames.HLD/blob/cd7630f613ac844dba579fc56f30d5048c73032d/wwwroot/data/MAP004.JSON.
    const MAP_4: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/test-data/maps/Map004.json"
    ));

    #[test]
    fn map_1() {
        let map: Map = serde_json::from_str(MAP_1).expect("failed to parse");
        assert!(!map.autoplay_bgm);
        // dbg!(map);

        let map_ser = serde_json::to_string(&map).expect("failed to serialize");
        let map_de = serde_json::from_str(&map_ser).expect("failed to parse");

        assert!(map == map_de);
    }

    #[test]
    fn map_2() {
        let map: Map = serde_json::from_str(MAP_2).expect("failed to parse");
        assert!(!map.autoplay_bgm);
        // dbg!(map);

        let map_ser = serde_json::to_string(&map).expect("failed to serialize");
        let map_de = serde_json::from_str(&map_ser).expect("failed to parse");

        assert!(map == map_de);
    }

    #[test]
    fn map_4() {
        let map: Map = serde_json::from_str(MAP_4).expect("failed to parse");
        assert!(!map.autoplay_bgm);
        // dbg!(map);

        let map_ser = serde_json::to_string(&map).expect("failed to serialize");
        let map_de = serde_json::from_str(&map_ser).expect("failed to parse");

        assert!(map == map_de);
    }
}
