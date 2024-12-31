use super::EventCommand;

/// Common event
#[derive(Debug, serde::Deserialize, serde::Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct CommonEvent {
    /// The id
    pub id: u32,

    /// The list of commands
    pub list: Vec<EventCommand>,

    /// The name
    pub name: String,

    /// ?
    #[serde(rename = "switchId")]
    pub switch_id: u32,

    /// ?
    pub trigger: u8,
}
