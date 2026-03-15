/// An event command
#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct EventCommand {
    /// The event code
    pub code: u32,

    /// The event indent
    pub indent: u16,

    /// The event parameters
    pub parameters: Vec<serde_json::Value>,
    
    /// I'm not sure if this is part of the core engine or not.
    ///
    /// This is only for MZ games.
    pub collapsed: Option<bool>,
}
