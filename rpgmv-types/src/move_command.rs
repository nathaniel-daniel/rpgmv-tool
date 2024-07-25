/// A move command
#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Clone)]
#[serde(deny_unknown_fields)]
pub struct MoveCommand {
    /// ?
    pub code: u32,

    /// ?
    pub parameters: Option<Vec<serde_json::Value>>,

    /// ?
    pub indent: Option<u32>,
}
