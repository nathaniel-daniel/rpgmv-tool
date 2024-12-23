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