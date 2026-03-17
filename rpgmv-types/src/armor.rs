#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Clone)]
#[serde(deny_unknown_fields)]
pub struct ArmorTrait {
    /// ?
    pub code: u32,

    /// ?
    #[serde(rename = "dataId")]
    pub data_id: u32,

    /// ?
    pub value: f32,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Clone)]
#[serde(deny_unknown_fields)]
pub struct Armor {
    /// The troop id
    pub id: u32,

    /// ?
    #[serde(rename = "atypeId")]
    pub a_type_id: u32,

    /// The armor description
    pub description: String,

    /// ?
    #[serde(rename = "etypeId")]
    pub e_type_id: u32,

    /// ?
    pub traits: Vec<ArmorTrait>,

    /// ?
    #[serde(rename = "iconIndex")]
    pub icon_index: u32,

    /// The armor name
    pub name: String,

    /// ?
    pub note: String,

    /// ?
    pub params: Vec<i32>,

    /// The cost of the armor.
    pub price: u32,
}
