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
    (102, SHOW_CHOICES),

    (108, COMMENT),

    (111, CONDITONAL_BRANCH),

    (117, COMMON_EVENT),

    (121, CONTROL_SWITCHES),
    (122, CONTROL_VARIABLES),
    (123, CONTROL_SELF_SWITCH),

    (201, TRANSFER_PLAYER),

    (205, SET_MOVEMENT_ROUTE),

    (222, FADEIN_SCREEN),

    (235, ERASE_PICTURE),

    (241, PLAY_BGM),

    (401, TEXT_DATA),

    (408, COMMENT_EXTRA),

    (411, ELSE),
    /// I think this is an end for the CONDITONAL_BRANCH block.
    /// I can't be sure as the game doesn't actually care if this exists;
    /// it just ignores it, only taking into account indents.
    (412, CONDITONAL_BRANCH_END),

    /// This is likely related to move routes somehow,
    /// like how the TEXT_DATA command extends the SHOW_TEXT command.
    /// However, I can't find the implementation of this instruction.
    /// Furthermore, I don't know why it's even included;
    /// Its data always duplicates the data of the SET_MOVEMENT_ROUTE command.
    (505, SET_MOVEMENT_ROUTE_EXTRA),
}

impl std::fmt::Debug for CommandCode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self.as_str() {
            Some(s) => write!(f, "{s}"),
            None => write!(f, "Unknown({})", self.0),
        }
    }
}
