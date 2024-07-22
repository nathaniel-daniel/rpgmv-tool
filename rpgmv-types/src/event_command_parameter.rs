use crate::AudioFile;
use crate::MoveCommand;
use crate::MoveRoute;

/// An event command parameter
#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Clone)]
#[serde(deny_unknown_fields, untagged)]
pub enum EventCommandParameter {
    /// A string
    String(String),

    /// A signed integer
    Int(i32),

    /// A boolean
    Bool(bool),

    /// A move route
    MoveRoute(MoveRoute),

    /// A move command
    MoveCommand(MoveCommand),

    /// An audio file
    AudioFile(AudioFile),
}

impl EventCommandParameter {
    /// Get this as an int.
    pub fn as_int(&self) -> Option<&i32> {
        match self {
            Self::Int(n) => Some(n),
            _ => None,
        }
    }
}
