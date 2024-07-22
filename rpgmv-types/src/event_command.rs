use crate::EventCommandParameter;

/// An event command
#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct EventCommand {
    /// The event code
    pub code: u32,

    /// The event indent
    pub indent: u16,

    /// The event parameters
    pub parameters: Vec<EventCommandParameter>,
}
