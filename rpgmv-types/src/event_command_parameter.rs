/// An event command parameter
#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(deny_unknown_fields, untagged)]
pub enum EventCommandParameter {
    /// A string
    String(String),

    /// A number
    Int(u32),

    /// A boolean
    Bool(bool),
}
