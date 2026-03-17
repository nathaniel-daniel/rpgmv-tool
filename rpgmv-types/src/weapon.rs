#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct WeaponTrait {
    /// ?
    pub code: u32,

    /// ?
    #[serde(rename = "dataId")]
    pub data_id: u32,

    /// ?
    pub value: f32,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Weapon {
    /// The weapon id
    pub id: u32,

    /// ?
    #[serde(rename = "animationId")]
    pub animation_id: u32,

    /// The weapon description
    pub description: String,

    /// ?
    #[serde(rename = "etypeId")]
    pub e_type_id: u32,

    /// ?
    pub traits: Vec<WeaponTrait>,

    /// ?
    #[serde(rename = "iconIndex")]
    pub icon_index: u32,

    /// The weapon name
    pub name: String,

    /// ?
    pub note: String,

    /// ?
    pub params: Vec<u32>,

    /// The weapon's price
    pub price: u32,

    /// ?
    #[serde(rename = "wtypeId")]
    pub w_type_id: u32,
}
