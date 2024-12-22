use super::Command;
use super::ParamReader;

impl Command {
    pub(super) fn parse_show_text(
        event_command: &rpgmv_types::EventCommand,
    ) -> anyhow::Result<Self> {
        let reader = ParamReader::new(event_command);
        reader.ensure_len_is(4)?;

        let face_name = reader.read_at(0, "face_name")?;
        let face_index = reader.read_at(1, "face_index")?;
        let background = reader.read_at(2, "background")?;
        let position_type = reader.read_at(3, "position_type")?;

        Ok(Command::ShowText {
            face_name,
            face_index,
            background,
            position_type,
            lines: Vec::new(),
        })
    }
}
