use crate::EventCommand;
use crate::EventPageCondition;
use crate::ImageFile;
use crate::MoveRoute;

/// An event page
#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct EventPage {
    /// ?
    pub conditions: EventPageCondition,

    /// ?
    #[serde(rename = "directionFix")]
    pub direction_fix: bool,

    /// ?
    pub image: ImageFile,

    /// The list of commands to execute
    pub list: Vec<EventCommand>,

    /// ?
    #[serde(rename = "moveFrequency")]
    pub move_frequency: u32,

    /// ?
    #[serde(rename = "moveRoute")]
    pub move_route: MoveRoute,

    /// ?
    #[serde(rename = "moveSpeed")]
    pub move_speed: u32,

    /// ?
    #[serde(rename = "moveType")]
    pub move_type: u32,

    /// ?
    #[serde(rename = "priorityType")]
    pub priority_type: u32,

    /// ?
    #[serde(rename = "stepAnime")]
    pub step_anime: bool,

    /// ?
    pub through: bool,

    /// ?
    pub trigger: u8,

    /// ?
    #[serde(rename = "walkAnime")]
    pub walk_anime: bool,
}
