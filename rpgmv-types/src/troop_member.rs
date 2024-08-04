/// A troop member
#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct TroopMember {
    /// The enemy id?
    #[serde(rename = "enemyId")]
    pub enemy_id: u32,

    /// The x coord?
    pub x: u32,

    /// The y coord?
    pub y: u32,

    /// Whether this is hidden?
    pub hidden: bool,
}
