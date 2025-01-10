#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Clone)]
#[serde(deny_unknown_fields)]
pub struct AudioFile {
    /// The file name
    pub name: String,

    /// ?
    pub pan: i8,

    /// ?
    pub pitch: u8,

    /// The volume
    pub volume: u8,
}
