mod function_call_writer;

use self::function_call_writer::FunctionCallWriter;
use self::function_call_writer::Ident;
use super::Command;
use super::ConditionalBranchCommand;
use super::Config;
use super::ControlVariablesValue;
use super::ControlVariablesValueGameData;
use super::MaybeRef;
use std::io::Write;

pub fn commands2py<W>(
    config: &Config,
    commands: &[(u16, Command)],
    mut writer: W,
) -> anyhow::Result<()>
where
    W: Write,
{
    for (indent, command) in commands.iter() {
        command2py(config, *indent, command, &mut writer)?;
    }

    Ok(())
}

fn command2py<W>(
    config: &Config,
    indent: u16,
    command: &Command,
    mut writer: W,
) -> anyhow::Result<()>
where
    W: Write,
{
    match command {
        Command::Nop => {}
        Command::ShowText {
            face_name,
            face_index,
            background,
            position_type,
            speaker_name,
            lines,
        } => {
            let mut writer = FunctionCallWriter::new(&mut writer, indent, "show_text")?;
            writer.write_param("face_name", face_name)?;
            writer.write_param("face_index", face_index)?;
            writer.write_param("background", background)?;
            writer.write_param("position_type", position_type)?;
            writer.write_param("speaker_name", speaker_name)?;
            writer.write_param("lines", lines)?;
            writer.finish()?;
        }
        Command::ShowChoices {
            choices,
            cancel_type,
            default_type,
            position_type,
            background,
        } => {
            let mut writer = FunctionCallWriter::new(&mut writer, indent, "show_choices")?;
            writer.write_param("choices", choices)?;
            writer.write_param("cancel_type", cancel_type)?;
            writer.write_param("default_type", default_type)?;
            writer.write_param("position_type", position_type)?;
            writer.write_param("background", background)?;
            writer.finish()?;
        }
        Command::Comment { lines } => {
            for line in lines.iter() {
                write_indent(&mut writer, indent)?;
                writeln!(&mut writer, "# {line}")?;
            }
        }
        Command::ConditionalBranch(command) => {
            write_indent(&mut writer, indent)?;
            write!(&mut writer, "if ")?;
            match command {
                ConditionalBranchCommand::Switch { id, check_true } => {
                    let name = config.get_switch_name(*id);
                    let check_true_str = if *check_true { "" } else { "not " };
                    writeln!(&mut writer, "{check_true_str}{name}:")?;
                }
                ConditionalBranchCommand::Variable {
                    lhs_id,
                    rhs_id,
                    operation,
                } => {
                    let lhs = config.get_variable_name(*lhs_id);
                    let rhs = match rhs_id {
                        MaybeRef::Constant(value) => value.to_string(),
                        MaybeRef::Ref(id) => config.get_variable_name(*id),
                    };
                    let operation = operation.as_str();

                    writeln!(&mut writer, "{lhs} {operation} {rhs}:")?;
                }
                ConditionalBranchCommand::SelfSwitch { name, check_true } => {
                    let name = escape_string(name);
                    let check_true_str = if *check_true { "" } else { "not " };
                    writeln!(&mut writer, "{check_true_str}game_self_switches.get(map_id=self.map_id, event_id=self.event_id, name='{name}'):")?;
                }
                ConditionalBranchCommand::ActorInParty { actor_id } => {
                    let actor_name = config.get_actor_name(*actor_id);

                    writeln!(
                        &mut writer,
                        "game_party.members.contains(actor={actor_name}):"
                    )?;
                }
                ConditionalBranchCommand::Timer { value, is_gte } => {
                    let cmp = if *is_gte { ">=" } else { "<=" };

                    writeln!(&mut writer, "game_timer.seconds() {cmp} {value}:")?;
                }
                ConditionalBranchCommand::ActorSkill { actor_id, skill_id } => {
                    let actor_name = config.get_actor_name(*actor_id);
                    let skill_name = config.get_skill_name(*skill_id);

                    writeln!(&mut writer, "{actor_name}.has_skill(skill={skill_name}):")?;
                }
                ConditionalBranchCommand::ActorArmor { actor_id, armor_id } => {
                    let actor_name = config.get_actor_name(*actor_id);
                    let armor_name = config.get_armor_name(*armor_id);

                    writeln!(&mut writer, "{actor_name}.has_armor(armor={armor_name}):")?;
                }
                ConditionalBranchCommand::ActorState { actor_id, state_id } => {
                    let actor_name = config.get_actor_name(*actor_id);
                    let state_name = config.get_state_name(*state_id);

                    writeln!(&mut writer, "{actor_name}.has_state(state={state_name}):")?;
                }
                ConditionalBranchCommand::EnemyState {
                    enemy_index,
                    state_id,
                } => {
                    let name = config.get_state_name(*state_id);

                    writeln!(
                        &mut writer,
                        "game_troop.members[{enemy_index}].is_state_affected(state={name}):"
                    )?;
                }
                ConditionalBranchCommand::Character {
                    character_id,
                    direction,
                } => {
                    let name = if *character_id < 0 {
                        "game_player".to_string()
                    } else {
                        format!("game_character_{character_id}")
                    };

                    writeln!(&mut writer, "{name}.direction == {direction}:")?;
                }
                ConditionalBranchCommand::Gold { value, check } => {
                    let check = check.as_str();

                    writeln!(&mut writer, "game_party.gold {check} {value}:")?;
                }
                ConditionalBranchCommand::Item { item_id } => {
                    let name = config.get_item_name(*item_id);

                    writeln!(&mut writer, "game_party.has_item(item={name}):")?;
                }
                ConditionalBranchCommand::Script { value } => {
                    let value = escape_string(value);

                    writeln!(&mut writer, "execute_script('{value}'):")?;
                }
            }
        }
        Command::CommonEvent { id } => {
            let name = config.get_common_event_name(*id);
            FunctionCallWriter::new(&mut writer, indent, &name)?.finish()?;
        }
        Command::Label { name } => {
            let mut writer = FunctionCallWriter::new(&mut writer, indent, "set_label")?;
            writer.set_multiline(false);
            writer.write_param("name", name)?;
            writer.finish()?;
        }
        Command::JumpToLabel { name } => {
            let mut writer = FunctionCallWriter::new(&mut writer, indent, "jump_to_label")?;
            writer.set_multiline(false);
            writer.write_param("name", name)?;
            writer.finish()?;
        }
        Command::ControlVariables {
            start_variable_id,
            end_variable_id,
            operation,
            value,
        } => {
            let operation = operation.as_str();
            let value = match value {
                ControlVariablesValue::Constant { value } => value.to_string(),
                ControlVariablesValue::Variable { id } => config.get_variable_name(*id),
                ControlVariablesValue::Random { start, stop } => {
                    format!("random.randrange(start={start}, stop={stop})")
                }
                ControlVariablesValue::GameData(game_data) => match game_data {
                    ControlVariablesValueGameData::NumItems { item_id } => {
                        let name = config.get_item_name(*item_id);

                        format!("game_party.get_num_items(item={name})")
                    }
                    ControlVariablesValueGameData::ActorLevel { actor_id } => {
                        let name = config.get_actor_name(*actor_id);
                        format!("{name}.level")
                    }
                    ControlVariablesValueGameData::ActorHp { actor_id } => {
                        let name = config.get_actor_name(*actor_id);
                        format!("{name}.hp")
                    }
                    ControlVariablesValueGameData::ActorMp { actor_id } => {
                        let name = config.get_actor_name(*actor_id);
                        format!("{name}.mp")
                    }
                    ControlVariablesValueGameData::CharacterMapX { character_id } => {
                        format!("game.get_character(id={character_id}).map_x")
                    }
                    ControlVariablesValueGameData::CharacterMapY { character_id } => {
                        format!("game.get_character(id={character_id}).map_y")
                    }
                    ControlVariablesValueGameData::MapId => "game_map.map_id()".to_string(),
                    ControlVariablesValueGameData::Gold => "game_party.gold".to_string(),
                    ControlVariablesValueGameData::Steps => "game_party.steps".to_string(),
                },
            };
            for variable_id in *start_variable_id..(*end_variable_id + 1) {
                let name = config.get_variable_name(variable_id);

                write_indent(&mut writer, indent)?;
                writeln!(&mut writer, "{name} {operation} {value}")?;
            }
        }
        Command::ControlSwitches {
            start_id,
            end_id,
            value,
        } => {
            for id in *start_id..(*end_id + 1) {
                let name = config.get_switch_name(id);
                let value = stringify_bool(*value);

                write_indent(&mut writer, indent)?;
                writeln!(&mut writer, "{name} = {value}")?;
            }
        }
        Command::ControlSelfSwitch { key, value } => {
            let value = stringify_bool(*value);

            write_indent(&mut writer, indent)?;
            writeln!(&mut writer, "game_self_switches['{key}'] = {value}")?;
        }
        Command::TransferPlayer {
            map_id,
            x,
            y,
            direction,
            fade_type,
        } => {
            let mut writer = FunctionCallWriter::new(&mut writer, indent, "transfer_player")?;
            match map_id {
                MaybeRef::Constant(id) => {
                    let name = format!("game_map_{id}");
                    writer.write_param("map", &Ident(&name))?;
                }
                MaybeRef::Ref(id) => {
                    let name = config.get_variable_name(*id);
                    writer.write_param("map_id", &Ident(&name))?;
                }
            }

            match x {
                MaybeRef::Constant(value) => {
                    writer.write_param("x", value)?;
                }
                MaybeRef::Ref(id) => {
                    let name = config.get_variable_name(*id);
                    writer.write_param("x", &Ident(&name))?;
                }
            }

            match y {
                MaybeRef::Constant(value) => {
                    writer.write_param("y", value)?;
                }
                MaybeRef::Ref(id) => {
                    let name = config.get_variable_name(*id);
                    writer.write_param("y", &Ident(&name))?;
                }
            }

            writer.write_param("direction", direction)?;
            writer.write_param("fade_type", fade_type)?;

            writer.finish()?;
        }
        Command::SetMovementRoute {
            character_id,
            route,
        } => {
            let mut writer = FunctionCallWriter::new(&mut writer, indent, "set_movement_route")?;
            writer.write_param("character_id", character_id)?;
            writer.write_param("route", route)?;
            writer.finish()?;
        }
        Command::ShowAnimation {
            character_id,
            animation_id,
            wait,
        } => {
            let mut writer = FunctionCallWriter::new(&mut writer, indent, "show_animation")?;
            writer.set_multiline(false);
            writer.write_param("character_id", character_id)?;
            writer.write_param("animation_id", animation_id)?;
            writer.write_param("wait", wait)?;
            writer.finish()?;
        }
        Command::ShowBalloonIcon {
            character_id,
            balloon_id,
            wait,
        } => {
            let mut writer = FunctionCallWriter::new(&mut writer, indent, "show_balloon_icon")?;
            writer.write_param("character_id", character_id)?;
            writer.write_param("balloon_id", balloon_id)?;
            writer.write_param("wait", wait)?;
            writer.set_multiline(false);
            writer.finish()?;
        }
        Command::FadeoutScreen => {
            FunctionCallWriter::new(&mut writer, indent, "fadeout_screen")?.finish()?;
        }
        Command::FadeinScreen => {
            FunctionCallWriter::new(&mut writer, indent, "fadein_screen")?.finish()?;
        }
        Command::ShakeScreen {
            power,
            speed,
            duration,
            wait,
        } => {
            let mut writer = FunctionCallWriter::new(&mut writer, indent, "shake_screen")?;
            writer.set_multiline(false);
            writer.write_param("power", power)?;
            writer.write_param("speed", speed)?;
            writer.write_param("duration", duration)?;
            writer.write_param("wait", wait)?;
            writer.finish()?;
        }
        Command::Wait { duration } => {
            let mut writer = FunctionCallWriter::new(&mut writer, indent, "wait")?;
            writer.set_multiline(false);
            writer.write_param("duration", duration)?;
            writer.finish()?;
        }
        Command::ErasePicture { picture_id } => {
            let mut writer = FunctionCallWriter::new(&mut writer, indent, "erase_picture")?;
            writer.set_multiline(false);
            writer.write_param("picture_id", picture_id)?;
            writer.finish()?;
        }
        Command::PlayBgm { audio } => {
            let mut writer = FunctionCallWriter::new(&mut writer, indent, "play_bgm")?;
            writer.write_param("audio", audio)?;
            writer.finish()?;
        }
        Command::FadeoutBgm { duration } => {
            let mut writer = FunctionCallWriter::new(&mut writer, indent, "fadeout_bgm")?;
            writer.set_multiline(false);
            writer.write_param("duration", duration)?;
            writer.finish()?;
        }
        Command::PlaySe { audio } => {
            let mut writer = FunctionCallWriter::new(&mut writer, indent, "play_se")?;
            writer.write_param("audio", audio)?;
            writer.finish()?;
        }
        Command::NameInputProcessing { actor_id, max_len } => {
            let actor = config.get_actor_name(*actor_id);

            let mut writer = FunctionCallWriter::new(&mut writer, indent, "name_input_processing")?;
            writer.set_multiline(false);
            writer.write_param("actor", &Ident(&actor))?;
            writer.write_param("max_len", max_len)?;
            writer.finish()?;
        }
        Command::When {
            choice_index,
            choice_name,
        } => {
            write_indent(&mut writer, indent)?;
            writeln!(
                &mut writer,
                "if get_choice_index() == {choice_index}: # {choice_name}"
            )?;
        }
        Command::WhenEnd => {
            // Trust indents over end commands
        }
        Command::Else => {
            write_indent(&mut writer, indent)?;
            writeln!(&mut writer, "else:")?;
        }
        Command::ConditionalBranchEnd => {
            // Trust indents over end commands
        }
        Command::Unknown { code, parameters } => {
            write_indent(&mut writer, indent)?;
            writeln!(
                &mut writer,
                "# Unknown Command Code {code:?}, parameters: {parameters:?}"
            )?;
        }
    }

    Ok(())
}

fn stringify_bool(b: bool) -> &'static str {
    match b {
        true => "True",
        false => "False",
    }
}

fn write_indent<W>(mut writer: W, indent: u16) -> std::io::Result<()>
where
    W: Write,
{
    for _ in 0..indent {
        write!(writer, "\t")?;
    }

    Ok(())
}

fn escape_string(input: &str) -> String {
    input.replace('\'', "\\'")
}
