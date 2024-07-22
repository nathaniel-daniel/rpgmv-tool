use crate::MoveCommand;

/// A move route
#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Clone)]
#[serde(deny_unknown_fields)]
pub struct MoveRoute {
    /// ?
    pub list: Vec<MoveCommand>,

    /// ?
    pub repeat: bool,

    /// ?
    pub skippable: bool,

    /// ?
    pub wait: bool,
}
