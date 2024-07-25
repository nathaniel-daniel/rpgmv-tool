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

#[cfg(test)]
mod test {
    use super::*;
    // Taken from https://github.com/craftadria/Timetrollergames.HLD/blob/cd7630f613ac844dba579fc56f30d5048c73032d/wwwroot/data/CommonEvents.json.
    const COMMON_EVENTS_1: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/test-data/common-events/CommonEvents1.json"
    ));

    #[test]
    fn common_events_1() {
        let common_events: Vec<Option<CommonEvent>> =
            serde_json::from_str(COMMON_EVENTS_1).expect("failed to parse");
        // dbg!(common_events);

        let common_events_ser = serde_json::to_string(&common_events).expect("failed to serialize");
        let common_events_de: Vec<Option<CommonEvent>> =
            serde_json::from_str(&common_events_ser).expect("failed to parse");

        assert!(common_events == common_events_de);
    }
}
