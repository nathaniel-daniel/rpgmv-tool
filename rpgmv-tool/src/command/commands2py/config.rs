use serde::de::Error;
use std::collections::BTreeMap;
use std::path::Path;

/// Config
#[derive(Debug, serde::Deserialize, Default)]
pub struct Config {
    /// Switches
    #[serde(default, deserialize_with = "deserialize_u32_key_btree_map")]
    pub switches: BTreeMap<u32, String>,

    /// Variables
    #[serde(default, deserialize_with = "deserialize_u32_key_btree_map")]
    pub variables: BTreeMap<u32, String>,

    /// Common Events
    #[serde(
        default,
        rename = "common-events",
        deserialize_with = "deserialize_u32_key_btree_map"
    )]
    pub common_events: BTreeMap<u32, String>,

    /// Actors
    #[serde(default, deserialize_with = "deserialize_u32_key_btree_map")]
    pub actors: BTreeMap<u32, String>,

    /// Skills
    #[serde(default, deserialize_with = "deserialize_u32_key_btree_map")]
    pub skills: BTreeMap<u32, String>,

    /// Items
    #[serde(default, deserialize_with = "deserialize_u32_key_btree_map")]
    pub items: BTreeMap<u32, String>,

    /// States
    #[serde(default, deserialize_with = "deserialize_u32_key_btree_map")]
    pub states: BTreeMap<u32, String>,

    /// Troops
    #[serde(default, deserialize_with = "deserialize_u32_key_btree_map")]
    pub troops: BTreeMap<u32, String>,

    /// Armors
    #[serde(default, deserialize_with = "deserialize_u32_key_btree_map")]
    pub armors: BTreeMap<u32, String>,
}

impl Config {
    /// Load this from a path.
    pub fn from_path<P>(path: P) -> anyhow::Result<Self>
    where
        P: AsRef<Path>,
    {
        let data = std::fs::read_to_string(path)?;
        let data: Self = toml::from_str(&data)?;
        Ok(data)
    }

    /// Get a switch name
    pub fn get_switch_name(&self, id: u32) -> String {
        self.switches
            .get(&id)
            .map(|name| name.to_string())
            .unwrap_or_else(|| format!("game_switch_{id}"))
    }

    /// Get a variable name
    pub fn get_variable_name(&self, id: u32) -> String {
        self.variables
            .get(&id)
            .map(|name| name.to_string())
            .unwrap_or_else(|| format!("game_variable_{id}"))
    }

    /// Get a common event name
    pub fn get_common_event_name(&self, id: u32) -> String {
        self.common_events
            .get(&id)
            .map(|name| name.to_string())
            .unwrap_or_else(|| format!("common_event_{id}"))
    }

    /// Get an actor name
    pub fn get_actor_name(&self, id: u32) -> String {
        self.actors
            .get(&id)
            .map(|name| name.to_string())
            .unwrap_or_else(|| format!("game_actor_{id}"))
    }

    /// Get a skill name
    pub fn get_skill_name(&self, id: u32) -> String {
        self.skills
            .get(&id)
            .map(|name| name.to_string())
            .unwrap_or_else(|| format!("game_skill_{id}"))
    }

    /// Get an item name
    pub fn get_item_name(&self, id: u32) -> String {
        self.items
            .get(&id)
            .map(|name| name.to_string())
            .unwrap_or_else(|| format!("game_item_{id}"))
    }

    /// Get a state name
    pub fn get_state_name(&self, id: u32) -> String {
        self.states
            .get(&id)
            .map(|name| name.to_string())
            .unwrap_or_else(|| format!("game_state_{id}"))
    }

    /// Get a troop name
    pub fn get_troop_name(&self, id: u32) -> String {
        self.troops
            .get(&id)
            .map(|name| name.to_string())
            .unwrap_or_else(|| format!("game_troop_{id}"))
    }

    /// Get an armor name
    pub fn get_armor_name(&self, id: u32) -> String {
        self.armors
            .get(&id)
            .map(|name| name.to_string())
            .unwrap_or_else(|| format!("game_armor_{id}"))
    }
}

fn deserialize_u32_key_btree_map<'de, D, V>(deserializer: D) -> Result<BTreeMap<u32, V>, D::Error>
where
    D: serde::Deserializer<'de>,
    V: serde::Deserialize<'de>,
{
    let map: BTreeMap<String, V> = serde::Deserialize::deserialize(deserializer)?;

    map.into_iter()
        .map(|(key, value)| {
            let key: u32 = key.parse().map_err(D::Error::custom)?;
            Ok((key, value))
        })
        .collect()
}
