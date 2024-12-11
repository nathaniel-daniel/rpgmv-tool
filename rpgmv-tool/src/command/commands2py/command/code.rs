/// A command code
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct CommandCode(pub u32);

impl CommandCode {
    /// This has no implementation.
    pub const NOP: Self = Self(0);

    pub const SHOW_TEXT: Self = Self(101);
    pub const SHOW_CHOICES: Self = Self(102);
    pub const INPUT_NUMBER: Self = Self(103);
    pub const SELECT_ITEM: Self = Self(104);
    pub const SHOW_SCROLLING_TEXT: Self = Self(105);

    pub const COMMENT: Self = Self(108);

    pub const CONDITONAL_BRANCH: Self = Self(111);

    pub const EXIT_EVENT_PROCESSING: Self = Self(115);

    pub const COMMON_EVENT: Self = Self(117);
    pub const LABEL: Self = Self(118);
    pub const JUMP_TO_LABEL: Self = Self(119);

    pub const CONTROL_SWITCHES: Self = Self(121);
    pub const CONTROL_VARIABLES: Self = Self(122);
    pub const CONTROL_SELF_SWITCH: Self = Self(123);
    pub const CONTROL_TIMER: Self = Self(124);
    pub const CHANGE_GOLD: Self = Self(125);
    pub const CHANGE_ITEMS: Self = Self(126);

    pub const CHANGE_ARMORS: Self = Self(128);
    pub const CHANGE_PARTY_MEMBER: Self = Self(129);

    pub const CHANGE_SAVE_ACCESS: Self = Self(134);

    pub const CHANGE_ENCOUNTER: Self = Self(136);

    pub const TRANSFER_PLAYER: Self = Self(201);

    pub const SET_EVENT_LOCATION: Self = Self(203);
    pub const SCROLL_MAP: Self = Self(204);
    pub const SET_MOVEMENT_ROUTE: Self = Self(205);

    pub const CHANGE_TRANSPARENCY: Self = Self(211);
    pub const SHOW_ANIMATION: Self = Self(212);
    pub const SHOW_BALLOON_ICON: Self = Self(213);
    pub const ERASE_EVENT: Self = Self(214);

    pub const CHANGE_PLAYER_FOLLOWERS: Self = Self(216);

    pub const FADEOUT_SCREEN: Self = Self(221);
    pub const FADEIN_SCREEN: Self = Self(222);
    pub const TINT_SCREEN: Self = Self(223);
    pub const FLASH_SCREEN: Self = Self(224);
    pub const SHAKE_SCREEN: Self = Self(225);

    pub const WAIT: Self = Self(230);
    pub const SHOW_PICTURE: Self = Self(231);
    pub const MOVE_PICTURE: Self = Self(232);

    pub const ERASE_PICTURE: Self = Self(235);

    pub const PLAY_BGM: Self = Self(241);
    pub const FADEOUT_BGM: Self = Self(242);
    pub const SAVE_BGM: Self = Self(243);
    pub const RESUME_BGM: Self = Self(244);
    pub const PLAY_BGS: Self = Self(245);
    pub const FADEOUT_BGS: Self = Self(246);

    pub const PLAY_SE: Self = Self(250);

    pub const BATTLE_PROCESSING: Self = Self(301);
    pub const SHOP_PROCESSING: Self = Self(302);
    pub const NAME_INPUT_PROCESSING: Self = Self(303);

    pub const CHANGE_HP: Self = Self(311);
    pub const CHANGE_MP: Self = Self(312);
    pub const CHANGE_STATE: Self = Self(313);
    pub const RECOVER_ALL: Self = Self(314);
    pub const CHANGE_EXP: Self = Self(315);
    pub const CHANGE_LEVEL: Self = Self(316);
    pub const CHANGE_PARAMETER: Self = Self(317);
    pub const CHANGE_SKILL: Self = Self(318);
    pub const CHANGE_EQUIPMENT: Self = Self(319);

    pub const CHANGE_CLASS: Self = Self(321);
    pub const CHANGE_ACTOR_IMAGES: Self = Self(322);

    pub const CHANGE_ENEMY_HP: Self = Self(331);

    pub const CHANGE_ENEMY_STATE: Self = Self(333);

    pub const FORCE_ACTION: Self = Self(339);
    pub const ABORT_BATTLE: Self = Self(340);

    pub const OPEN_SAVE_SCREEN: Self = Self(352);
    pub const GAME_OVER: Self = Self(353);

    pub const RETURN_TO_TITLE_SCREEN: Self = Self(354);
    pub const SCRIPT: Self = Self(355);
    pub const PLUGIN_COMMAND: Self = Self(356);

    pub const TEXT_DATA: Self = Self(401);
    pub const WHEN: Self = Self(402);
    pub const WHEN_CANCEL: Self = Self(403);
    /// I think this is an end for the WHEN block.
    /// I can't be sure as the game doesn't actually care if this exists;
    /// it just ignores it, only taking into account indents.
    pub const WHEN_END: Self = Self(404);
    pub const SHOW_SCROLLING_TEXT_EXTRA: Self = Self(405);

    pub const ELSE: Self = Self(411);
    /// I think this is an end for the CONDITONAL_BRANCH block.
    /// I can't be sure as the game doesn't actually care if this exists;
    /// it just ignores it, only taking into account indents.
    pub const CONDITONAL_BRANCH_END: Self = Self(412);

    /// This is likely related to move routes somehow,
    /// like how the TEXT_DATA command extends the SHOW_TEXT command.
    /// However, I can't find the implementation of this instruction.
    /// Furthermore, I don't know why it's even included;
    /// Its data always duplicates the data of the SET_MOVEMENT_ROUTE command.
    pub const SET_MOVEMENT_ROUTE_EXTRA: Self = Self(505);

    pub const IF_WIN: Self = Self(601);
    pub const IF_ESCAPE: Self = Self(602);
    pub const IF_LOSE: Self = Self(603);
    /// I think this is an end for an IF_WIN, IF_ESCAPE, or IF_LOSE block.
    /// I can't be sure as the game doesn't actually care if this exists;
    /// it just ignores it, only taking into account indents.
    pub const BATTLE_RESULT_END: Self = Self(604);
    pub const SHOP_PROCESSING_EXTRA: Self = Self(605);
}

impl std::fmt::Debug for CommandCode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Self::NOP => write!(f, "NOP"),
            Self::SHOW_TEXT => write!(f, "SHOW_TEXT"),
            Self::SHOW_CHOICES => write!(f, "SHOW_CHOICES"),
            Self::INPUT_NUMBER => write!(f, "INPUT_NUMBER"),
            Self::SELECT_ITEM => write!(f, "SELECT_ITEM"),
            Self::SHOW_SCROLLING_TEXT => write!(f, "SHOW_SCROLLING_TEXT"),
            Self::COMMENT => write!(f, "COMMENT"),
            Self::CONDITONAL_BRANCH => write!(f, "CONDITONAL_BRANCH"),
            Self::COMMON_EVENT => write!(f, "COMMON_EVENT"),
            Self::LABEL => write!(f, "LABEL"),
            Self::JUMP_TO_LABEL => write!(f, "JUMP_TO_LABEL"),
            Self::CONTROL_SWITCHES => write!(f, "CONTROL_SWITCHES"),
            Self::CONTROL_VARIABLES => write!(f, "CONTROL_VARIABLES"),
            Self::CONTROL_SELF_SWITCH => write!(f, "CONTROL_SELF_SWITCH"),
            Self::CONTROL_TIMER => write!(f, "CONTROL_TIMER"),
            Self::CHANGE_GOLD => write!(f, "CHANGE_GOLD"),
            Self::CHANGE_ITEMS => write!(f, "CHANGE_ITEMS"),
            Self::CHANGE_ARMORS => write!(f, "CHANGE_ARMORS"),
            Self::CHANGE_PARTY_MEMBER => write!(f, "CHANGE_PARTY_MEMBER"),
            Self::CHANGE_SAVE_ACCESS => write!(f, "CHANGE_SAVE_ACCESS"),
            Self::CHANGE_ENCOUNTER => write!(f, "CHANGE_ENCOUNTER"),
            Self::SET_EVENT_LOCATION => write!(f, "SET_EVENT_LOCATION"),
            Self::SCROLL_MAP => write!(f, "SCROLL_MAP"),
            Self::TRANSFER_PLAYER => write!(f, "TRANSFER_PLAYER"),
            Self::SET_MOVEMENT_ROUTE => write!(f, "SET_MOVEMENT_ROUTE"),
            Self::CHANGE_TRANSPARENCY => write!(f, "CHANGE_TRANSPARENCY"),
            Self::SHOW_ANIMATION => write!(f, "SHOW_ANIMATION"),
            Self::SHOW_BALLOON_ICON => write!(f, "SHOW_BALLOON_ICON"),
            Self::ERASE_EVENT => write!(f, "ERASE_EVENT"),
            Self::CHANGE_PLAYER_FOLLOWERS => write!(f, "CHANGE_PLAYER_FOLLOWERS"),
            Self::FADEOUT_SCREEN => write!(f, "FADEOUT_SCREEN"),
            Self::FADEIN_SCREEN => write!(f, "FADEIN_SCREEN"),
            Self::TINT_SCREEN => write!(f, "TINT_SCREEN"),
            Self::FLASH_SCREEN => write!(f, "FLASH_SCREEN"),
            Self::SHAKE_SCREEN => write!(f, "SHAKE_SCREEN"),
            Self::WAIT => write!(f, "WAIT"),
            Self::SHOW_PICTURE => write!(f, "SHOW_PICTURE"),
            Self::MOVE_PICTURE => write!(f, "MOVE_PICTURE"),
            Self::ERASE_PICTURE => write!(f, "ERASE_PICTURE"),
            Self::PLAY_BGM => write!(f, "PLAY_BGM"),
            Self::FADEOUT_BGM => write!(f, "FADEOUT_BGM"),
            Self::SAVE_BGM => write!(f, "SAVE_BGM"),
            Self::RESUME_BGM => write!(f, "RESUME_BGM"),
            Self::FADEOUT_BGS => write!(f, "FADEOUT_BGS"),
            Self::PLAY_BGS => write!(f, "PLAY_BGS"),
            Self::PLAY_SE => write!(f, "PLAY_SE"),
            Self::BATTLE_PROCESSING => write!(f, "BATTLE_PROCESSING"),
            Self::SHOP_PROCESSING => write!(f, "SHOP_PROCESSING"),
            Self::NAME_INPUT_PROCESSING => write!(f, "NAME_INPUT_PROCESSING"),
            Self::CHANGE_HP => write!(f, "CHANGE_HP"),
            Self::CHANGE_MP => write!(f, "CHANGE_MP"),
            Self::CHANGE_STATE => write!(f, "CHANGE_STATE"),
            Self::RECOVER_ALL => write!(f, "RECOVER_ALL"),
            Self::CHANGE_EXP => write!(f, "CHANGE_EXP"),
            Self::CHANGE_LEVEL => write!(f, "CHANGE_LEVEL"),
            Self::CHANGE_PARAMETER => write!(f, "CHANGE_PARAMETER"),
            Self::CHANGE_SKILL => write!(f, "CHANGE_SKILL"),
            Self::CHANGE_EQUIPMENT => write!(f, "CHANGE_EQUIPMENT"),
            Self::CHANGE_CLASS => write!(f, "CHANGE_CLASS"),
            Self::CHANGE_ACTOR_IMAGES => write!(f, "CHANGE_ACTOR_IMAGES"),
            Self::CHANGE_ENEMY_HP => write!(f, "CHANGE_ENEMY_HP"),
            Self::CHANGE_ENEMY_STATE => write!(f, "CHANGE_ENEMY_STATE"),
            Self::FORCE_ACTION => write!(f, "FORCE_ACTION"),
            Self::ABORT_BATTLE => write!(f, "ABORT_BATTLE"),
            Self::OPEN_SAVE_SCREEN => write!(f, "OPEN_SAVE_SCREEN"),
            Self::GAME_OVER => write!(f, "GAME_OVER"),
            Self::RETURN_TO_TITLE_SCREEN => write!(f, "RETURN_TO_TITLE_SCREEN"),
            Self::SCRIPT => write!(f, "SCRIPT"),
            Self::PLUGIN_COMMAND => write!(f, "PLUGIN_COMMAND"),
            Self::TEXT_DATA => write!(f, "TEXT_DATA"),
            Self::WHEN => write!(f, "WHEN"),
            Self::WHEN_CANCEL => write!(f, "WHEN_CANCEL"),
            Self::WHEN_END => write!(f, "WHEN_END"),
            Self::SHOW_SCROLLING_TEXT_EXTRA => write!(f, "SHOW_SCROLLING_TEXT_EXTRA"),
            Self::ELSE => write!(f, "ELSE"),
            Self::CONDITONAL_BRANCH_END => write!(f, "CONDITONAL_BRANCH_END"),
            Self::SET_MOVEMENT_ROUTE_EXTRA => write!(f, "SET_MOVEMENT_ROUTE_EXTRA"),
            Self::IF_WIN => write!(f, "IF_WIN"),
            Self::IF_ESCAPE => write!(f, "IF_ESCAPE"),
            Self::IF_LOSE => write!(f, "IF_LOSE"),
            Self::BATTLE_RESULT_END => write!(f, "BATTLE_RESULT_END"),
            Self::SHOP_PROCESSING_EXTRA => write!(f, "SHOP_PROCESSING_EXTRA"),
            _ => write!(f, "Unknown({})", self.0),
        }
    }
}
