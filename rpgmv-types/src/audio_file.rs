#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AudioFile {
    /// The file name
    pub name: String,

    /// ?
    pub pan: u8,

    /// ?
    pub pitch: u8,

    /// The volume
    pub volume: u8,
}
