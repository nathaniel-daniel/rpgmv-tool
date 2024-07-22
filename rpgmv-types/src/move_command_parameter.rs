/// A move command parameter
#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Clone)]
#[serde(deny_unknown_fields, untagged)]
pub enum MoveCommandParameter {
    /// A signed integer
    Int(i32),
}
