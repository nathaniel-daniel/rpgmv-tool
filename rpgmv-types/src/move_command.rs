use crate::MoveCommandParameter;

/// A move command
#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct MoveCommand {
    /// ?
    pub code: u32,

    /// ?
    pub parameters: Option<Vec<MoveCommandParameter>>,

    /// ?
    pub indent: Option<u32>,
}
