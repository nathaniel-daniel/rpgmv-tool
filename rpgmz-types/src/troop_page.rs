use super::EventCommand;
use super::TroopPageCondition;

/// A troop page
#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct TroopPage {
    /// Troop page conditions
    pub conditions: TroopPageCondition,

    /// The commands
    pub list: Vec<EventCommand>,

    /// ?
    pub span: u32,
}
