macro_rules! command_codes {
    (
        $(
            $(#[$docs:meta])*
            ($id:expr, $name:ident),
        )+
    ) => {
        impl CommandCode {
            $(
                $(#[$docs])*
                pub const $name: Self = Self($id);
            )+

            pub fn as_str(self) -> Option<&'static str> {
                match self {
                    $(
                    Self::$name => Some(stringify!($name)),
                    )+
                    _ => None
                }
            }
        }
    }
}

/// A command code
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct CommandCode(pub u32);

command_codes! {
    /// This has no implementation.
    (0, NOP),

    (101, SHOW_TEXT),

    (108, COMMENT),

    (117, COMMON_EVENT),

    (122, CONTROL_VARIABLES),
    (123, CONTROL_SELF_SWITCH),

    (222, FADEIN_SCREEN),

    (401, TEXT_DATA),

    (408, COMMENT_EXTRA),
}

impl std::fmt::Debug for CommandCode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self.as_str() {
            Some(s) => write!(f, "{s}"),
            None => write!(f, "Unknown({})", self.0),
        }
    }
}
