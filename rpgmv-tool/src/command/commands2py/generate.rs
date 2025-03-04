mod function_call_writer;

use self::function_call_writer::FunctionCallWriter;
use self::function_call_writer::Ident;
use super::Command;
use super::ConditionalBranchCommand;
use super::Config;
use super::ControlVariablesValue;
use super::ControlVariablesValueGameData;
use super::GetLocationInfoKind;
use super::MaybeRef;
use anyhow::ensure;
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
            lines,
        } => {
            let mut writer = FunctionCallWriter::new(&mut writer, indent, "show_text")?;
            writer.write_param("face_name", face_name)?;
            writer.write_param("face_index", face_index)?;
            writer.write_param("background", background)?;
            writer.write_param("position_type", position_type)?;
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
        Command::ShowScrollingText {
            speed,
            no_fast,
            lines,
        } => {
            let no_fast = stringify_bool(*no_fast);

            write_indent(&mut writer, indent)?;
            writeln!(&mut writer, "show_scrolling_text(")?;

            write_indent(&mut writer, indent + 1)?;
            writeln!(&mut writer, "speed={speed},")?;

            write_indent(&mut writer, indent + 1)?;
            writeln!(&mut writer, "no_fast={no_fast},")?;

            write_indent(&mut writer, indent + 1)?;
            writeln!(&mut writer, "lines=[")?;

            for line in lines {
                let line = escape_string(line);

                write_indent(&mut writer, indent + 2)?;
                writeln!(&mut writer, "'{line}',")?;
            }

            write_indent(&mut writer, indent + 1)?;
            writeln!(&mut writer, "],")?;

            write_indent(&mut writer, indent)?;
            writeln!(&mut writer, ")")?;
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
                ConditionalBranchCommand::Button { key_name } => {
                    let key_name = escape_string(key_name);

                    writeln!(&mut writer, "game_input.is_pressed(key_name='{key_name}'):")?;
                }
                ConditionalBranchCommand::Script { value } => {
                    let value = escape_string(value);

                    writeln!(&mut writer, "execute_script('{value}'):")?;
                }
            }
        }
        Command::Loop => {
            write_indent(&mut writer, indent)?;
            writeln!(&mut writer, "while True:")?;
        }
        Command::ExitEventProcessing => {
            write_indent(&mut writer, indent)?;
            writeln!(&mut writer, "exit_event_processing()")?;
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
                    ControlVariablesValueGameData::ActorParam { param_index } => {
                        format!("game_param_{param_index}")
                    }
                    ControlVariablesValueGameData::CharacterMapX { character_id } => {
                        format!("game.get_character(id={character_id}).map_x")
                    }
                    ControlVariablesValueGameData::CharacterMapY { character_id } => {
                        format!("game.get_character(id={character_id}).map_y")
                    }
                    ControlVariablesValueGameData::CharacterScreenX { character_id } => {
                        format!("game.get_character(id={character_id}).screen_x")
                    }
                    ControlVariablesValueGameData::CharacterScreenY { character_id } => {
                        format!("game.get_character(id={character_id}).screen_y")
                    }
                    ControlVariablesValueGameData::MapId => "game_map.map_id()".to_string(),
                    ControlVariablesValueGameData::Gold => "game_party.gold".to_string(),
                    ControlVariablesValueGameData::Steps => "game_party.steps".to_string(),
                },
                ControlVariablesValue::Script { value } => {
                    let value = escape_string(value);
                    format!("execute_script('{value}')")
                }
            };
            for variable_id in *start_variable_id..(*end_variable_id + 1) {
                let name = config.get_variable_name(variable_id);

                write_indent(&mut writer, indent)?;
                writeln!(&mut writer, "{name} {operation} {value}")?;
            }
        }
        Command::ControlSelfSwitch { key, value } => {
            let value = stringify_bool(*value);

            write_indent(&mut writer, indent)?;
            writeln!(&mut writer, "game_self_switches['{key}'] = {value}")?;
        }
        Command::ControlTimer { start_seconds } => {
            write_indent(&mut writer, indent)?;
            match start_seconds {
                Some(start_seconds) => {
                    writeln!(&mut writer, "game_timer.start(seconds={start_seconds})")?
                }
                None => writeln!(&mut writer, "game_timer.stop()")?,
            }
        }
        Command::ChangeGold { is_add, value } => {
            let op = if *is_add { "+=" } else { "-=" };
            let value = match value {
                MaybeRef::Constant(value) => value.to_string(),
                MaybeRef::Ref(id) => config.get_variable_name(*id),
            };

            write_indent(&mut writer, indent)?;
            writeln!(&mut writer, "game_party.gold {op} {value}")?;
        }
        Command::ChangeItems {
            item_id,
            is_add,
            value,
        } => {
            let item = config.get_item_name(*item_id);
            let sign = if *is_add { "" } else { "-" };
            let value = match value {
                MaybeRef::Constant(value) => value.to_string(),
                MaybeRef::Ref(id) => config.get_variable_name(*id),
            };
            let value = format!("{sign}{value}");

            let mut writer = FunctionCallWriter::new(&mut writer, indent, "gain_item")?;
            writer.set_multiline(false);
            writer.write_param("item", &Ident(&item))?;
            writer.write_param("value", &Ident(&value))?;
            writer.finish()?;
        }
        Command::ChangeArmors {
            armor_id,
            is_add,
            value,
            include_equipped,
        } => {
            let armor = config.get_armor_name(*armor_id);
            let sign = if *is_add { "" } else { "-" };
            let value = match value {
                MaybeRef::Constant(value) => value.to_string(),
                MaybeRef::Ref(id) => config.get_variable_name(*id),
            };
            let value = format!("{sign}{value}");

            let mut writer = FunctionCallWriter::new(&mut writer, indent, "gain_armor")?;
            writer.set_multiline(false);
            writer.write_param("armor", &Ident(&armor))?;
            writer.write_param("value", &Ident(&value))?;
            writer.write_param("include_equipped", include_equipped)?;
            writer.finish()?;
        }
        Command::ChangePartyMember {
            actor_id,
            is_add,
            initialize,
        } => {
            let actor_name = config.get_actor_name(*actor_id);
            let fn_name = if *is_add {
                "add_party_member"
            } else {
                "remove_party_member"
            };

            let mut writer = FunctionCallWriter::new(&mut writer, indent, fn_name)?;
            writer.set_multiline(false);
            writer.write_param("actor", &Ident(&actor_name))?;
            // The argument is always provided, but ignored by remove ops.
            if *is_add {
                writer.write_param("initialize", initialize)?;
            }
            writer.finish()?;
        }
        Command::ChangeSaveAccess { disable } => {
            let fn_name = if *disable {
                "disable_saving"
            } else {
                "enable_saving"
            };

            let mut writer = FunctionCallWriter::new(&mut writer, indent, fn_name)?;
            writer.finish()?;
        }
        Command::SetEventLocation {
            character_id,
            x,
            y,
            direction,
        } => {
            let mut writer = FunctionCallWriter::new(&mut writer, indent, "set_event_location")?;
            writer.write_param("character_id", character_id)?;
            match x {
                MaybeRef::Constant(x) => {
                    writer.write_param("x", x)?;
                }
                MaybeRef::Ref(x) => {
                    let x = config.get_variable_name(*x);
                    writer.write_param("x", &Ident(&x))?;
                }
            }
            match y {
                MaybeRef::Constant(y) => {
                    writer.write_param("y", y)?;
                }
                MaybeRef::Ref(y) => {
                    let y = config.get_variable_name(*y);
                    writer.write_param("y", &Ident(&y))?;
                }
            }
            if let Some(direction) = direction {
                writer.write_param("direction", direction)?;
            }
            writer.finish()?;
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
        Command::ChangeTransparency { set_transparent } => {
            let set_transparent = stringify_bool(*set_transparent);

            write_indent(&mut writer, indent)?;
            writeln!(
                &mut writer,
                "change_transparency(set_transparent={set_transparent})"
            )?
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
            writer.set_multiline(false);
            writer.write_param("character_id", character_id)?;
            writer.write_param("balloon_id", balloon_id)?;
            writer.write_param("wait", wait)?;
            writer.finish()?;
        }
        Command::ChangePlayerFollowers { is_show } => {
            let fn_name = if *is_show {
                "show_player_followers"
            } else {
                "hide_player_followers"
            };

            write_indent(&mut writer, indent)?;
            writeln!(&mut writer, "{fn_name}()")?
        }
        Command::FadeoutScreen => {
            FunctionCallWriter::new(&mut writer, indent, "fadeout_screen")?.finish()?;
        }
        Command::FadeinScreen => {
            FunctionCallWriter::new(&mut writer, indent, "fadein_screen")?.finish()?;
        }
        Command::TintScreen {
            tone,
            duration,
            wait,
        } => {
            let wait = stringify_bool(*wait);

            write_indent(&mut writer, indent)?;
            writeln!(
                &mut writer,
                "tint_screen(tone={tone:?}, duration={duration}, wait={wait})"
            )?
        }
        Command::FlashScreen {
            color,
            duration,
            wait,
        } => {
            let wait = stringify_bool(*wait);

            write_indent(&mut writer, indent)?;
            writeln!(
                &mut writer,
                "flash_screen(color={color:?}, duration={duration}, wait={wait})"
            )?
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
        Command::ShowPicture {
            picture_id,
            picture_name,
            origin,
            x,
            y,
            scale_x,
            scale_y,
            opacity,
            blend_mode,
        } => {
            let picture_name = escape_string(picture_name);
            let x = match x {
                MaybeRef::Constant(value) => value.to_string(),
                MaybeRef::Ref(id) => config.get_variable_name(*id),
            };
            let y = match y {
                MaybeRef::Constant(value) => value.to_string(),
                MaybeRef::Ref(id) => config.get_variable_name(*id),
            };

            write_indent(&mut writer, indent)?;
            writeln!(&mut writer, "show_picture(")?;

            write_indent(&mut writer, indent + 1)?;
            writeln!(&mut writer, "picture_id={picture_id},")?;

            write_indent(&mut writer, indent + 1)?;
            writeln!(&mut writer, "picture_name='{picture_name}',")?;

            write_indent(&mut writer, indent + 1)?;
            writeln!(&mut writer, "origin={origin},")?;

            write_indent(&mut writer, indent + 1)?;
            writeln!(&mut writer, "x={x},")?;

            write_indent(&mut writer, indent + 1)?;
            writeln!(&mut writer, "y={y},")?;

            write_indent(&mut writer, indent + 1)?;
            writeln!(&mut writer, "scale_x={scale_x},")?;

            write_indent(&mut writer, indent + 1)?;
            writeln!(&mut writer, "scale_y={scale_y},")?;

            write_indent(&mut writer, indent + 1)?;
            writeln!(&mut writer, "opacity={opacity},")?;

            write_indent(&mut writer, indent + 1)?;
            writeln!(&mut writer, "blend_mode={blend_mode},")?;

            write_indent(&mut writer, indent)?;
            writeln!(&mut writer, ")")?;
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
        Command::SaveBgm => {
            write_indent(&mut writer, indent)?;
            writeln!(&mut writer, "save_bgm()")?;
        }
        Command::ResumeBgm => {
            write_indent(&mut writer, indent)?;
            writeln!(&mut writer, "resume_bgm()")?;
        }
        Command::PlayBgs { audio } => {
            write_indent(&mut writer, indent)?;
            writeln!(&mut writer, "play_bgs(")?;

            write_indent(&mut writer, indent + 1)?;
            write!(&mut writer, "audio=")?;
            write_audio_file(&mut writer, indent + 1, audio)?;

            write_indent(&mut writer, indent)?;
            writeln!(&mut writer, ")")?;
        }
        Command::FadeoutBgs { duration } => {
            write_indent(&mut writer, indent)?;
            writeln!(&mut writer, "fadeout_bgs(duration={duration})")?;
        }
        Command::PlaySe { audio } => {
            let mut writer = FunctionCallWriter::new(&mut writer, indent, "play_se")?;
            writer.write_param("audio", audio)?;
            writer.finish()?;
        }
        Command::GetLocationInfo {
            variable_id,
            kind,
            x,
            y,
        } => {
            let variable = config.get_variable_name(*variable_id);
            let x = match x {
                MaybeRef::Constant(x) => x.to_string(),
                MaybeRef::Ref(x) => config.get_variable_name(*x),
            };
            let y = match y {
                MaybeRef::Constant(y) => y.to_string(),
                MaybeRef::Ref(y) => config.get_variable_name(*y),
            };

            let value = match kind {
                GetLocationInfoKind::TerrainTag => {
                    format!("game_map.get_terrain_tag(x={x}, y={y})")
                }
                GetLocationInfoKind::EventId => {
                    format!("game_map.get_event_id(x={x}, y={y})")
                }
            };

            write_indent(&mut writer, indent)?;
            writeln!(&mut writer, "{variable} = {value}")?;
        }
        Command::BattleProcessing {
            troop_id,
            can_escape,
            can_lose,
        } => {
            let mut writer = FunctionCallWriter::new(&mut writer, indent, "battle_processing")?;
            match troop_id {
                Some(MaybeRef::Constant(id)) => {
                    let name = config.get_troop_name(*id);
                    writer.write_param("troop", &Ident(&name))?;
                }
                Some(MaybeRef::Ref(id)) => {
                    let name = config.get_variable_name(*id);
                    writer.write_param("troop_id", &Ident(&name))?;
                }
                None => {
                    writer.write_param("troop_id", &Ident("game.random_encounter_troop_id()"))?;
                }
            }
            writer.write_param("can_escape", can_escape)?;
            writer.write_param("can_lose", can_lose)?;
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
        Command::ChangeHp {
            actor_id,
            is_add,
            value,
            allow_death,
        } => {
            let mut writer = FunctionCallWriter::new(&mut writer, indent, "gain_hp")?;
            writer.set_multiline(false);
            match actor_id {
                MaybeRef::Constant(actor_id) => {
                    let name = config.get_actor_name(*actor_id);
                    writer.write_param("actor", &Ident(&name))?;
                }
                MaybeRef::Ref(variable_id) => {
                    let name = config.get_variable_name(*variable_id);
                    writer.write_param("actor_id", &Ident(&name))?;
                }
            };
            let sign = if *is_add { "" } else { "-" };
            let value = match value {
                MaybeRef::Constant(value) => value.to_string(),
                MaybeRef::Ref(id) => config.get_variable_name(*id),
            };
            let value = format!("{sign}{value}");
            writer.write_param("value", &Ident(&value))?;
            writer.write_param("allow_death", allow_death)?;
            writer.finish()?;
        }
        Command::ChangeMp {
            actor_id,
            is_add,
            value,
        } => {
            let mut writer = FunctionCallWriter::new(&mut writer, indent, "gain_mp")?;
            writer.set_multiline(false);
            match actor_id {
                MaybeRef::Constant(actor_id) => {
                    let name = config.get_actor_name(*actor_id);
                    writer.write_param("actor", &Ident(&name))?;
                }
                MaybeRef::Ref(variable_id) => {
                    let name = config.get_variable_name(*variable_id);
                    writer.write_param("actor_id", &Ident(&name))?;
                }
            };
            let sign = if *is_add { "" } else { "-" };
            let value = match value {
                MaybeRef::Constant(value) => value.to_string(),
                MaybeRef::Ref(id) => config.get_variable_name(*id),
            };
            let value = format!("{sign}{value}");

            writer.write_param("value", &Ident(&value))?;
            writer.finish()?;
        }
        Command::ChangeState {
            actor_id,
            is_add_state,
            state_id,
        } => {
            let actor_arg = match actor_id {
                MaybeRef::Constant(0) => "actors=game_party".to_string(),
                MaybeRef::Constant(actor_id) => {
                    let name = config.get_actor_name(*actor_id);
                    format!("actor={name}")
                }
                MaybeRef::Ref(variable_id) => {
                    let name = config.get_variable_name(*variable_id);
                    format!("actor_id={name}")
                }
            };

            let fn_name = if *is_add_state {
                "add_state"
            } else {
                "remove_state"
            };
            let state = config.get_state_name(*state_id);

            write_indent(&mut writer, indent)?;
            writeln!(&mut writer, "{fn_name}({actor_arg}, state={state})")?;
        }
        Command::ChangeLevel {
            actor_id,
            is_add,
            value,
            show_level_up,
        } => {
            let actor_arg = match actor_id {
                MaybeRef::Constant(actor_id) => {
                    let name = config.get_actor_name(*actor_id);
                    format!("actor={name}")
                }
                MaybeRef::Ref(variable_id) => {
                    let name = config.get_variable_name(*variable_id);
                    format!("actor_id={name}")
                }
            };
            let sign = if *is_add { "" } else { "-" };
            let value = match value {
                MaybeRef::Constant(value) => value.to_string(),
                MaybeRef::Ref(id) => config.get_variable_name(*id),
            };
            let show_level_up = stringify_bool(*show_level_up);

            write_indent(&mut writer, indent)?;
            writeln!(
                &mut writer,
                "gain_level({actor_arg}, value={sign}{value}, show_level_up={show_level_up})"
            )?;
        }
        Command::ChangeSkill {
            actor_id,
            is_learn_skill,
            skill_id,
        } => {
            let actor_arg = match actor_id {
                MaybeRef::Constant(actor_id) => {
                    let name = config.get_actor_name(*actor_id);
                    format!("actor={name}")
                }
                MaybeRef::Ref(variable_id) => {
                    let name = config.get_variable_name(*variable_id);
                    format!("actor_id={name}")
                }
            };
            let fn_name = if *is_learn_skill {
                "learn_skill"
            } else {
                "forget_skill"
            };
            let skill = config.get_skill_name(*skill_id);

            write_indent(&mut writer, indent)?;
            writeln!(&mut writer, "{fn_name}({actor_arg}, skill={skill})")?;
        }
        Command::ChangeClass {
            actor_id,
            class_id,
            keep_exp,
        } => {
            let actor = config.get_actor_name(*actor_id);
            let class = config.get_class_name(*class_id);
            let keep_exp = stringify_bool(*keep_exp);

            write_indent(&mut writer, indent)?;
            writeln!(
                &mut writer,
                "change_class(actor={actor}, klass={class}, keep_exp={keep_exp})"
            )?;
        }
        Command::ChangeActorImages {
            actor_id,
            character_name,
            character_index,
            face_name,
            face_index,
            battler_name,
        } => {
            let actor_name = config.get_actor_name(*actor_id);
            let character_name = escape_string(character_name);
            let face_name = escape_string(face_name);
            let battler_name = escape_string(battler_name);

            write_indent(&mut writer, indent)?;
            writeln!(&mut writer, "change_actor_images(")?;

            write_indent(&mut writer, indent + 1)?;
            writeln!(&mut writer, "actor={actor_name},")?;

            write_indent(&mut writer, indent + 1)?;
            writeln!(&mut writer, "character_name='{character_name}',")?;

            write_indent(&mut writer, indent + 1)?;
            writeln!(&mut writer, "character_index={character_index},")?;

            write_indent(&mut writer, indent + 1)?;
            writeln!(&mut writer, "face_name='{face_name}',")?;

            write_indent(&mut writer, indent + 1)?;
            writeln!(&mut writer, "face_index={face_index},")?;

            write_indent(&mut writer, indent + 1)?;
            writeln!(&mut writer, "battler_name='{battler_name}',")?;

            write_indent(&mut writer, indent)?;
            writeln!(&mut writer, ")")?;
        }
        Command::ForceAction {
            is_enemy,
            id,
            skill_id,
            target_index,
        } => {
            let arg_0 = if *is_enemy {
                format!("enemy_index={id}")
            } else {
                let actor = config.get_actor_name(*id);
                format!("actor={actor}")
            };
            let skill = config.get_skill_name(*skill_id);

            write_indent(&mut writer, indent)?;
            writeln!(
                &mut writer,
                "force_action({arg_0}, skill={skill}, target_index={target_index})"
            )?;
        }
        Command::AbortBattle => {
            let mut writer = FunctionCallWriter::new(&mut writer, indent, "abort_battle")?;
            writer.finish()?;
        }
        Command::GameOver => {
            let mut writer = FunctionCallWriter::new(&mut writer, indent, "game_over")?;
            writer.finish()?;
        }
        Command::ReturnToTitleScreen => {
            let mut writer =
                FunctionCallWriter::new(&mut writer, indent, "return_to_title_screen")?;
            writer.finish()?;
        }
        Command::Script { lines } => {
            write_indent(&mut writer, indent)?;
            writeln!(&mut writer, "script(")?;

            write_indent(&mut writer, indent + 1)?;
            writeln!(writer, "lines=[")?;

            for line in lines {
                let line = escape_string(line);

                write_indent(&mut writer, indent + 2)?;
                writeln!(writer, "'{line}',")?;
            }

            write_indent(&mut writer, indent + 1)?;
            writeln!(&mut writer, "],")?;

            write_indent(&mut writer, indent)?;
            writeln!(&mut writer, ")")?;
        }
        Command::PluginCommand { params } => {
            write_indent(&mut writer, indent)?;
            write!(&mut writer, "plugin_command(")?;
            for (i, param) in params.iter().enumerate() {
                if i != 0 {
                    write!(&mut writer, ", ")?;
                }
                let param = escape_string(param);
                write!(&mut writer, "'{param}'")?;
            }
            writeln!(&mut writer, ")")?;
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
        Command::WhenCancel {
            choice_index,
            choice_name,
        } => {
            ensure!(choice_name.is_none());

            write_indent(&mut writer, indent)?;
            writeln!(
                &mut writer,
                "if get_choice_index() == -1: # Cancel, index={choice_index}"
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
        Command::RepeatAbove => {
            // This is just a loop end
        }
        Command::IfWin => {
            writeln!(&mut writer, "if game_battle_result.is_win():")?;
        }
        Command::IfEscape => {
            writeln!(&mut writer, "if game_battle_result.is_escape():")?;
        }
        Command::IfLose => {
            writeln!(&mut writer, "if game_battle_result.is_lose():")?;
        }
        Command::BattleResultEnd => {
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

fn write_audio_file<W>(
    mut writer: W,
    indent: u16,
    audio: &rpgmv_types::AudioFile,
) -> std::io::Result<()>
where
    W: Write,
{
    let audio_name = escape_string(&audio.name);

    writeln!(&mut writer, "AudioFile(")?;

    write_indent(&mut writer, indent + 1)?;
    writeln!(&mut writer, "name='{audio_name}',")?;

    write_indent(&mut writer, indent + 1)?;
    writeln!(&mut writer, "pan={},", audio.pan)?;

    write_indent(&mut writer, indent + 1)?;
    writeln!(&mut writer, "pitch={},", audio.pitch)?;

    write_indent(&mut writer, indent + 1)?;
    writeln!(&mut writer, "volume={},", audio.volume)?;

    write_indent(&mut writer, indent)?;
    writeln!(&mut writer, "),")?;

    Ok(())
}
