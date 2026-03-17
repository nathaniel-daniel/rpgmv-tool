#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct SkillDamage {
    /// ?
    pub critical: bool,

    /// ?
    #[serde(rename = "elementId")]
    pub element_id: i32,

    /// ?
    pub formula: String,

    /// ?
    #[serde(rename = "type")]
    pub kind: u32,

    /// ?
    pub variance: u32,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct SkillEffect {
    /// ?
    pub code: u32,

    /// ?
    #[serde(rename = "dataId")]
    pub data_id: u32,

    /// ?
    pub value1: f32,

    /// ?
    pub value2: u32,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Skill {
    /// The skill id
    pub id: u32,

    /// The animation id
    #[serde(rename = "animationId")]
    pub animation_id: i32,

    /// ?
    pub damage: SkillDamage,

    /// The description of the skill.
    pub description: String,

    /// ?
    pub effects: Vec<SkillEffect>,

    /// ?
    #[serde(rename = "hitType")]
    pub hit_type: u32,

    /// ?
    #[serde(rename = "iconIndex")]
    pub icon_index: u32,

    /// ?
    pub message1: String,

    /// ?
    pub message2: String,

    /// ?
    #[serde(rename = "mpCost")]
    pub mp_cost: u32,

    /// The skill name
    pub name: String,

    /// ?
    pub note: String,

    /// ?
    pub occasion: u32,

    /// ?
    pub repeats: u32,

    /// ?
    #[serde(rename = "requiredWtypeId1")]
    pub required_w_type_id_1: u32,

    /// ?
    #[serde(rename = "requiredWtypeId2")]
    pub required_w_type_id_2: u32,

    /// ?
    pub scope: u32,

    /// ?
    pub speed: i32,

    /// ?
    #[serde(rename = "stypeId")]
    pub s_type_id: u32,

    /// ?
    #[serde(rename = "successRate")]
    pub success_rate: u32,

    /// ?
    #[serde(rename = "tpCost")]
    pub tp_cost: u32,

    /// ?
    #[serde(rename = "tpGain")]
    pub tp_gain: u32,

    /// ?
    ///
    /// This is an MZ-only field.
    #[serde(rename = "messageType")]
    pub message_type: Option<u32>,
}
