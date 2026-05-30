use std::collections::HashMap;

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct Plugin {
    /// The name of the plugin
    pub name: String,
    /// The status of the plugin, enabled or disabled.
    pub status: bool,
    /// The description of the plugin.
    pub description: String,

    /// Arguments passed the the plugin to configure its behavior.
    ///
    /// Arguments take the form of a key-value string map.
    /// As an example, the number 5 would be stringifed as "5" before being inserted into this map.
    pub parameters: HashMap<String, String>,
}
