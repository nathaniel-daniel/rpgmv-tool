use super::Command;
use super::MaybeRef;
use super::ParamReader;

#[derive(Debug, Copy, Clone)]
enum TroopIdKind {
    Constant,
    Variable,
    Random,
}

impl TroopIdKind {
    /// Get this from a u8.
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => Self::Constant,
            1 => Self::Variable,
            _ => Self::Random,
        }
    }
}

impl Command {
    pub(super) fn parse_battle_processing(
        event_command: &rpgmv_types::EventCommand,
    ) -> anyhow::Result<Self> {
        let reader = ParamReader::new(event_command);
        reader.ensure_len_is(4)?;

        let troop_id_kind = reader.read_at(0, "is_constant")?;
        let troop_id_kind = TroopIdKind::from_u8(troop_id_kind);
        let troop_id = reader.read_at(1, "troop_id")?;
        let troop_id = match troop_id_kind {
            TroopIdKind::Constant => Some(MaybeRef::Constant(troop_id)),
            TroopIdKind::Variable => Some(MaybeRef::Ref(troop_id)),
            TroopIdKind::Random => None,
        };
        let can_escape = reader.read_at(2, "can_escape")?;
        let can_lose = reader.read_at(3, "can_lose")?;

        Ok(Self::BattleProcessing {
            troop_id,
            can_escape,
            can_lose,
        })
    }
}
