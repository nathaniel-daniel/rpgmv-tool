use super::TroopMember;
use super::TroopPage;

/// A troop
#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Troop {
    /// The troop id
    pub id: u32,

    /// The troop members
    pub members: Vec<TroopMember>,

    /// The troop name
    pub name: String,

    /// Event pages
    pub pages: Vec<TroopPage>,
}
