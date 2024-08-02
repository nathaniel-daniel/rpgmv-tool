/// An encounter
#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Encounter {
    /// ?
    #[serde(rename = "regionSet")]
    pub region_set: Vec<()>,

    /// The troop id to spawn
    #[serde(rename = "troopId")]
    pub troop_id: u32,

    /// The relative weight of this event
    pub weight: u16,
}
