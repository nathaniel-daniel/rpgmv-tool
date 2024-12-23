mod function_call_writer;

use self::function_call_writer::FunctionCallWriter;
use super::Command;
use super::ConditionalBranchCommand;
use super::Config;
use super::ControlVariablesValue;
use super::ControlVariablesValueGameData;
use super::GetLocationInfoKind;
use super::MaybeRef;
use anyhow::bail;
use anyhow::ensure;
use anyhow::Context;
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
            write_indent(&mut writer, indent)?;
            writeln!(&mut writer, "show_choices(")?;

            write_indent(&mut writer, indent + 1)?;
            writeln!(&mut writer, "choices=[")?;

            for choice in choices {
                let choice = escape_string(choice);

                write_indent(&mut writer, indent + 2)?;
                writeln!(&mut writer, "'{choice}',")?;
            }

            write_indent(&mut writer, indent + 1)?;
            writeln!(&mut writer, "],")?;

            write_indent(&mut writer, indent + 1)?;
            writeln!(&mut writer, "cancel_type={cancel_type},")?;

            write_indent(&mut writer, indent + 1)?;
            writeln!(&mut writer, "default_type={default_type},")?;

            write_indent(&mut writer, indent + 1)?;
            writeln!(&mut writer, "position_type={position_type},")?;

            write_indent(&mut writer, indent + 1)?;
            writeln!(&mut writer, "background={background},")?;

            write_indent(&mut writer, indent)?;
            writeln!(&mut writer, ")")?;
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
        Command::Comment { comment } => {
            write_indent(&mut writer, indent)?;
            writeln!(&mut writer, "# {comment}")?;
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
        Command::ExitEventProcessing => {
            write_indent(&mut writer, indent)?;
            writeln!(&mut writer, "exit_event_processing()")?;
        }
        Command::CommonEvent { id } => {
            let name = config.get_common_event_name(*id);
            FunctionCallWriter::new(&mut writer, indent, &name)?.finish()?;
        }
        Command::Label { name } => {
            let name = escape_string(name);

            write_indent(&mut writer, indent)?;
            writeln!(&mut writer, "set_label('{name}')")?;
        }
        Command::JumpToLabel { name } => {
            let name = escape_string(name);

            write_indent(&mut writer, indent)?;
            writeln!(&mut writer, "jump_to_label('{name}')")?;
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

            write_indent(&mut writer, indent)?;
            writeln!(&mut writer, "gain_item(item={item}, value={sign}{value})")?;
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
            let include_equipped = stringify_bool(*include_equipped);

            write_indent(&mut writer, indent)?;
            writeln!(&mut writer, "gain_armor(item={armor}, value={sign}{value}, include_equipped={include_equipped})")?;
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
            let initialize = stringify_bool(*initialize);

            write_indent(&mut writer, indent)?;
            write!(&mut writer, "{fn_name}(actor={actor_name}")?;
            // The argument is always provided, but ignored by remove ops.
            if *is_add {
                write!(&mut writer, ", initialize={initialize}")?;
            }
            writeln!(&mut writer, ")")?;
        }
        Command::ChangeSaveAccess { disable } => {
            let fn_name = if *disable {
                "disable_saving"
            } else {
                "enable_saving"
            };
            write_indent(&mut writer, indent)?;
            writeln!(&mut writer, "{fn_name}()")?;
        }
        Command::SetEventLocation {
            character_id,
            x,
            y,
            direction,
        } => {
            let x = match x {
                MaybeRef::Constant(x) => x.to_string(),
                MaybeRef::Ref(x) => config.get_variable_name(*x),
            };
            let y = match y {
                MaybeRef::Constant(y) => y.to_string(),
                MaybeRef::Ref(y) => config.get_variable_name(*y),
            };

            write_indent(&mut writer, indent)?;
            write!(
                &mut writer,
                "set_event_location(character_id={character_id}, x={x}, y={y}"
            )?;
            if let Some(direction) = direction {
                write!(&mut writer, ", direction={direction}")?;
            }
            writeln!(&mut writer, ")")?;
        }
        Command::TransferPlayer {
            map_id,
            x,
            y,
            direction,
            fade_type,
        } => {
            let map_arg = match map_id {
                MaybeRef::Constant(id) => format!("map=game_map_{id}"),
                MaybeRef::Ref(id) => {
                    let name = config.get_variable_name(*id);
                    format!("map_id={name}")
                }
            };
            let x = match x {
                MaybeRef::Constant(value) => value.to_string(),
                MaybeRef::Ref(id) => config.get_variable_name(*id),
            };
            let y = match y {
                MaybeRef::Constant(value) => value.to_string(),
                MaybeRef::Ref(id) => config.get_variable_name(*id),
            };
            write_indent(&mut writer, indent)?;
            writeln!(&mut writer, "transfer_player({map_arg}, x={x}, y={y}, direction={direction}, fade_type={fade_type})")?;
        }
        Command::SetMovementRoute {
            character_id,
            route,
        } => {
            let repeat = stringify_bool(route.repeat);
            let skippable = stringify_bool(route.skippable);
            let wait = stringify_bool(route.wait);

            write_indent(&mut writer, indent)?;
            writeln!(&mut writer, "set_movement_route(")?;

            write_indent(&mut writer, indent + 1)?;
            writeln!(&mut writer, "character_id={character_id},")?;

            write_indent(&mut writer, indent + 1)?;
            writeln!(&mut writer, "route=MoveRoute(")?;

            write_indent(&mut writer, indent + 2)?;
            writeln!(&mut writer, "repeat={repeat},")?;

            write_indent(&mut writer, indent + 2)?;
            writeln!(&mut writer, "skippable={skippable},")?;

            write_indent(&mut writer, indent + 2)?;
            writeln!(&mut writer, "wait={wait},")?;

            write_indent(&mut writer, indent + 2)?;
            writeln!(&mut writer, "list=[")?;

            for command in route.list.iter() {
                let command_indent = command
                    .indent
                    .map(|indent| indent.to_string())
                    .unwrap_or_else(|| "None".to_string());

                write_indent(&mut writer, indent + 3)?;
                writeln!(&mut writer, "MoveCommand(")?;

                write_indent(&mut writer, indent + 4)?;
                writeln!(&mut writer, "code={},", command.code)?;

                write_indent(&mut writer, indent + 4)?;
                writeln!(&mut writer, "indent={command_indent},")?;

                match command.parameters.as_ref() {
                    Some(parameters) => {
                        write_indent(&mut writer, indent + 4)?;
                        writeln!(&mut writer, "parameters=[")?;

                        for parameter in parameters {
                            match parameter {
                                serde_json::Value::Number(number) if number.is_i64() => {
                                    let value = number.as_i64().context("value is not an i64")?;

                                    write_indent(&mut writer, indent + 5)?;
                                    writeln!(&mut writer, "{value},")?;
                                }
                                serde_json::Value::String(value) => {
                                    let value = escape_string(value);
                                    write_indent(&mut writer, indent + 5)?;
                                    writeln!(&mut writer, "'{value}',")?;
                                }
                                serde_json::Value::Object(object) => {
                                    write_indent(&mut writer, indent + 5)?;
                                    writeln!(&mut writer, "{{")?;

                                    for (key, value) in object.iter() {
                                        write_indent(&mut writer, indent + 6)?;
                                        writeln!(&mut writer, "'{key}': {value},")?;
                                    }

                                    write_indent(&mut writer, indent + 5)?;
                                    writeln!(&mut writer, "}},")?;
                                }
                                _ => {
                                    bail!("cannot write move route parameter \"{parameter:?}\"")
                                }
                            }
                        }

                        write_indent(&mut writer, indent + 4)?;
                        writeln!(&mut writer, "],")?;
                    }
                    None => {
                        write_indent(&mut writer, indent + 4)?;
                        writeln!(&mut writer, "parameters=None,")?;
                    }
                }

                write_indent(&mut writer, indent + 3)?;
                writeln!(&mut writer, "),")?;
            }

            write_indent(&mut writer, indent + 2)?;
            writeln!(&mut writer, "]")?;

            write_indent(&mut writer, indent + 1)?;
            writeln!(&mut writer, "),")?;

            write_indent(&mut writer, indent)?;
            writeln!(&mut writer, ")")?;
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
            let wait = stringify_bool(*wait);

            write_indent(&mut writer, indent)?;
            writeln!(&mut writer, "show_animation(character_id={character_id}, animation_id={animation_id}, wait={wait})")?
        }
        Command::ShowBalloonIcon {
            character_id,
            balloon_id,
            wait,
        } => {
            let wait = stringify_bool(*wait);

            write_indent(&mut writer, indent)?;
            writeln!(&mut writer, "show_balloon_icon(character_id={character_id}, balloon_id={balloon_id}, wait={wait})")?
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
            write_indent(&mut writer, indent)?;
            writeln!(&mut writer, "fadeout_screen()")?
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
            let wait = stringify_bool(*wait);

            write_indent(&mut writer, indent)?;
            writeln!(
                &mut writer,
                "shake_screen(power={power}, speed={speed}, duration={duration}, wait={wait})"
            )?
        }
        Command::Wait { duration } => {
            write_indent(&mut writer, indent)?;
            writeln!(&mut writer, "wait(duration={duration})")?
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
            write_indent(&mut writer, indent)?;
            writeln!(&mut writer, "play_bgm(")?;

            write_indent(&mut writer, indent + 1)?;
            write!(&mut writer, "audio=")?;
            write_audio_file(&mut writer, indent + 1, audio)?;

            write_indent(&mut writer, indent)?;
            writeln!(&mut writer, ")")?;
        }
        Command::FadeoutBgm { duration } => {
            write_indent(&mut writer, indent)?;
            writeln!(&mut writer, "fadeout_bgm(duration={duration})")?;
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
            write_indent(&mut writer, indent)?;
            writeln!(&mut writer, "play_se(")?;

            write_indent(&mut writer, indent + 1)?;
            write!(&mut writer, "audio=")?;
            write_audio_file(&mut writer, indent + 1, audio)?;

            write_indent(&mut writer, indent)?;
            writeln!(&mut writer, ")")?;
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
            let troop_arg = match troop_id {
                MaybeRef::Constant(id) => {
                    let name = config.get_troop_name(*id);
                    format!("troop={name}")
                }
                MaybeRef::Ref(id) => {
                    let name = config.get_variable_name(*id);

                    format!("troop_id={name}")
                }
            };
            let can_escape = stringify_bool(*can_escape);
            let can_lose = stringify_bool(*can_lose);

            write_indent(&mut writer, indent)?;
            writeln!(
                &mut writer,
                "battle_processing({troop_arg}, can_escape={can_escape}, can_lose={can_lose})"
            )?;
        }
        Command::NameInputProcessing { actor_id, max_len } => {
            let actor = config.get_actor_name(*actor_id);

            write_indent(&mut writer, indent)?;
            writeln!(
                &mut writer,
                "name_input_processing(actor={actor}, max_len={max_len})"
            )?;
        }
        Command::ChangeHp {
            actor_id,
            is_add,
            value,
            allow_death,
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
            let allow_death = stringify_bool(*allow_death);

            write_indent(&mut writer, indent)?;
            writeln!(
                &mut writer,
                "gain_hp({actor_arg}, value={sign}{value}, allow_death={allow_death})"
            )?;
        }
        Command::ChangeMp {
            actor_id,
            is_add,
            value,
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

            write_indent(&mut writer, indent)?;
            writeln!(&mut writer, "gain_mp({actor_arg}, value={sign}{value})")?;
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
            write_indent(&mut writer, indent)?;
            writeln!(&mut writer, "abort_battle()")?;
        }
        Command::ReturnToTitleScreen => {
            write_indent(&mut writer, indent)?;
            writeln!(&mut writer, "return_to_title_screen()")?;
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
