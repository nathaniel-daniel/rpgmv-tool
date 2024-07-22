/// An event page condition set
#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct EventPageCondition {
    /// The actor id
    #[serde(rename = "actorId")]
    pub actor_id: u32,

    /// Whether the actor id is valid
    #[serde(rename = "actorValid")]
    pub actor_valid: bool,

    /// The item id
    #[serde(rename = "itemId")]
    pub item_id: u32,

    /// Whether the item id is valid
    #[serde(rename = "itemValid")]
    pub item_valid: bool,

    /// ?
    #[serde(rename = "selfSwitchCh")]
    pub self_switch_ch: char,

    /// Whether the self switch ch is valid
    #[serde(rename = "selfSwitchValid")]
    pub self_switch_valid: bool,

    /// ?
    #[serde(rename = "switch1Id")]
    pub switch1_id: u32,

    /// Whether the switch1 id is valid
    #[serde(rename = "switch1Valid")]
    pub switch1_valid: bool,

    /// ?
    #[serde(rename = "switch2Id")]
    pub switch2_id: u32,

    /// Whether the switch2 id is valid
    #[serde(rename = "switch2Valid")]
    pub switch2_valid: bool,

    /// ?
    #[serde(rename = "variableId")]
    pub variable_id: u32,

    /// Whether the variable id is valid
    #[serde(rename = "variableValid")]
    pub variable_valid: bool,

    /// ?
    #[serde(rename = "variableValue")]
    pub variable_value: u32,
}
