/// An image
#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Clone)]
#[serde(deny_unknown_fields)]
pub struct ImageFile {
    /// ?
    #[serde(rename = "tileId")]
    pub tile_id: u32,

    /// ?
    #[serde(rename = "characterName")]
    pub character_name: String,

    /// ?
    pub direction: u8,

    /// ?
    pub pattern: u8,

    /// ?
    #[serde(rename = "characterIndex")]
    pub character_index: u8,
}
