/// A troop page condition
#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct TroopPageCondition {
    #[serde(rename = "actorHp")]
    pub actor_hp: u32,
    
    #[serde(rename = "actorId")]
    pub actor_id: u32,
    
    #[serde(rename = "actorValid")]
    pub actor_valid: bool,
    
    #[serde(rename = "enemyHp")]
    pub enemy_hp: u32,
    
     #[serde(rename = "enemyIndex")]
    pub enemy_index: u32,
    
    #[serde(rename = "enemyValid")]
    pub enemy_valid: bool,
    
    #[serde(rename = "switchId")]
    pub switch_id: u32,
    
    #[serde(rename = "switchValid")]
    pub switch_valid: bool,
    
    #[serde(rename = "turnA")]
    pub turn_a: u32,
    
    #[serde(rename = "turnB")]
    pub turn_b: u32,
    
    #[serde(rename = "turnEnding")]
    pub turn_ending: bool,
    
    #[serde(rename = "turnValid")]
    pub turn_valid: bool,
}
