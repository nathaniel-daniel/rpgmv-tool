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
    /// Args passed the the plugin to configure its behavior.
    pub parameters: HashMap<String, String>,
}
