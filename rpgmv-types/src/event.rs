use crate::EventPage;

/// An event
#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Event {
    /// The event id
    pub id: u32,

    /// The event name
    pub name: String,

    /// ?
    pub note: String,

    /// ?
    pub pages: Vec<EventPage>,

    /// ?
    pub x: u8,

    /// ?
    pub y: u8,
}
