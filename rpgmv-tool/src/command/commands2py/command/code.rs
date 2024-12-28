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
    (103, INPUT_NUMBER),
    (104, SELECT_ITEM),
    (105, SHOW_SCROLLING_TEXT),

    (108, COMMENT),

    (111, CONDITONAL_BRANCH),
    (112, LOOP),

    (115, EXIT_EVENT_PROCESSING),

    (117, COMMON_EVENT),
    (118, LABEL),
    (119, JUMP_TO_LABEL),

    (121, CONTROL_SWITCHES),
    (122, CONTROL_VARIABLES),
    (123, CONTROL_SELF_SWITCH),
    (124, CONTROL_TIMER),
    (125, CHANGE_GOLD),
    (126, CHANGE_ITEMS),

    (128, CHANGE_ARMORS),
    (129, CHANGE_PARTY_MEMBER),

    (134, CHANGE_SAVE_ACCESS),

    (136, CHANGE_ENCOUNTER),

    (201, TRANSFER_PLAYER),

    (203, SET_EVENT_LOCATION),
    (204, SCROLL_MAP),
    (205, SET_MOVEMENT_ROUTE),

    (211, CHANGE_TRANSPARENCY),
    (212, SHOW_ANIMATION),
    (213, SHOW_BALLOON_ICON),
    (214, ERASE_EVENT),

    (216, CHANGE_PLAYER_FOLLOWERS),

    (221, FADEOUT_SCREEN),
    (222, FADEIN_SCREEN),
    (223, TINT_SCREEN),
    (224, FLASH_SCREEN),
    (225, SHAKE_SCREEN),

    (230, WAIT),
    (231, SHOW_PICTURE),
    (232, MOVE_PICTURE),

    (235, ERASE_PICTURE),

    (241, PLAY_BGM),
    (242, FADEOUT_BGM),
    (243, SAVE_BGM),
    (244, RESUME_BGM),
    (245, PLAY_BGS),
    (246, FADEOUT_BGS),

    (250, PLAY_SE),

    (285, GET_LOCATION_INFO),

    (301, BATTLE_PROCESSING),
    (302, SHOP_PROCESSING),
    (303, NAME_INPUT_PROCESSING),

    (311, CHANGE_HP),
    (312, CHANGE_MP),
    (313, CHANGE_STATE),
    (314, RECOVER_ALL),
    (315, CHANGE_EXP),
    (316, CHANGE_LEVEL),
    (317, CHANGE_PARAMETER),
    (318, CHANGE_SKILL),
    (319, CHANGE_EQUIPMENT),

    (321, CHANGE_CLASS),
    (322, CHANGE_ACTOR_IMAGES),

    (331, CHANGE_ENEMY_HP),

    (333, CHANGE_ENEMY_STATE),

    (339, FORCE_ACTION),
    (340, ABORT_BATTLE),

    (352, OPEN_SAVE_SCREEN),
    (353, GAME_OVER),
    (354, RETURN_TO_TITLE_SCREEN),
    (355, SCRIPT),
    (356, PLUGIN_COMMAND),

    (401, TEXT_DATA),
    (402, WHEN),
    (403, WHEN_CANCEL),
    /// I think this is an end for the WHEN block.
    /// I can't be sure as the game doesn't actually care if this exists;
    /// it just ignores it, only taking into account indents.
    (404, WHEN_END),
    (405, SHOW_SCROLLING_TEXT_EXTRA),

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

    (601, IF_WIN),
    (602, IF_ESCAPE),
    (603, IF_LOSE),
    /// I think this is an end for an IF_WIN, IF_ESCAPE, or IF_LOSE block.
    /// I can't be sure as the game doesn't actually care if this exists;
    /// it just ignores it, only taking into account indents.
    (604, BATTLE_RESULT_END),
    (605, SHOP_PROCESSING_EXTRA),

    (655, SCRIPT_EXTRA),
}

impl std::fmt::Debug for CommandCode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self.as_str() {
            Some(s) => write!(f, "{s}"),
            None => write!(f, "Unknown({})", self.0),
        }
    }
}
