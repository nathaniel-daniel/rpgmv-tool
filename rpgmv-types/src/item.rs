#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Clone)]
#[serde(deny_unknown_fields)]
pub struct ItemDamage {
    /// ?
    pub critical: bool,

    /// ?
    #[serde(rename = "elementId")]
    pub element_id: u32,

    /// ?
    pub formula: String,

    /// ?
    #[serde(rename = "type")]
    pub kind: u32,

    /// ?
    pub variance: u32,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Clone)]
#[serde(deny_unknown_fields)]
pub struct ItemEffect {
    /// ?
    pub code: u32,

    #[serde(rename = "dataId")]
    pub data_id: u32,

    /// ?
    pub value1: f64,

    /// ?
    pub value2: u32,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Clone)]
#[serde(deny_unknown_fields)]
pub struct Item {
    /// The item id
    pub id: u32,

    /// ?
    #[serde(rename = "animationId")]
    pub animation_id: u32,

    /// ?
    pub consumable: bool,

    /// ?
    pub damage: ItemDamage,

    /// The item description
    pub description: String,

    /// ?
    ///
    /// The typing for this is incorrect, I have not encountered a sample where this is populated.
    pub effects: Vec<ItemEffect>,

    /// ?
    #[serde(rename = "hitType")]
    pub hit_type: u32,

    /// ?
    #[serde(rename = "iconIndex")]
    pub icon_index: u32,

    /// ?
    #[serde(rename = "itypeId")]
    pub i_type_id: u32,

    /// The item name
    pub name: String,

    /// ?
    pub note: String,

    /// ?
    pub occasion: u32,

    /// The item price
    pub price: u32,

    /// ?
    pub repeats: u32,

    /// ?
    pub scope: u32,

    /// ?
    pub speed: u32,

    /// ?
    #[serde(rename = "successRate")]
    pub success_rate: u32,

    /// ?
    #[serde(rename = "tpGain")]
    pub tp_gain: u32,
}
