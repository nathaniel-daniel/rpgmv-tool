use super::Command;
use super::ConditionalBranchCommand;
use super::Config;
use super::ControlVariablesValue;
use super::ControlVariablesValueGameData;
use super::MaybeRef;
use anyhow::bail;
use anyhow::Context;
use std::fmt::Write;

pub fn commands2py<W>(
    config: &Config,
    commands: &[(u16, Command)],
    mut writer: W,
) -> anyhow::Result<()>
where
    W: std::fmt::Write,
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
            write_indent(&mut writer, indent)?;
            writeln!(writer, "show_text(")?;

            write_indent(&mut writer, indent + 1)?;
            writeln!(writer, "face_name='{face_name}',")?;

            write_indent(&mut writer, indent + 1)?;
            writeln!(writer, "face_index={face_index},")?;

            write_indent(&mut writer, indent + 1)?;
            writeln!(writer, "background={background},")?;

            write_indent(&mut writer, indent + 1)?;
            writeln!(writer, "position_type={position_type},")?;

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
                ConditionalBranchCommand::ActorInParty { actor_id } => {
                    let actor_name = config.get_actor_name(*actor_id);

                    writeln!(
                        &mut writer,
                        "game_party.members.contains(actor={actor_name}):"
                    )?;
                }
                ConditionalBranchCommand::ActorArmor { actor_id, armor_id } => {
                    let actor_name = config.get_actor_name(*actor_id);
                    let armor_name = config.get_armor_name(*armor_id);

                    writeln!(&mut writer, "{actor_name}.has_armor(armor={armor_name}):")?;
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

            write_indent(&mut writer, indent)?;
            writeln!(&mut writer, "{name}()")?;
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
                    ControlVariablesValueGameData::Gold => "game_party.gold".to_string(),
                    _ => bail!("ControlVariablesValueGameData {game_data:?} is not supported"),
                },
            };
            for variable_id in *start_variable_id..(*end_variable_id + 1) {
                let name = config.get_variable_name(variable_id);
                write_indent(&mut writer, indent)?;
                writeln!(&mut writer, "{name} {operation} {value}")?;
            }
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
        Command::ChangeSaveAccess { disable } => {
            let fn_name = if *disable {
                "disable_saving"
            } else {
                "enable_saving"
            };
            write_indent(&mut writer, indent)?;
            writeln!(&mut writer, "{fn_name}()")?
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
        Command::ShowBalloonIcon {
            character_id,
            balloon_id,
            wait,
        } => {
            let wait = stringify_bool(*wait);

            write_indent(&mut writer, indent)?;
            writeln!(&mut writer, "show_balloon_icon(character_id={character_id}, balloon_id={balloon_id}, wait={wait})")?
        }
        Command::FadeoutScreen => {
            write_indent(&mut writer, indent)?;
            writeln!(&mut writer, "fadeout_screen()")?
        }
        Command::FadeinScreen => {
            write_indent(&mut writer, indent)?;
            writeln!(&mut writer, "fadein_screen()")?
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
            write_indent(&mut writer, indent)?;
            writeln!(&mut writer, "erase_picture(picture_id={picture_id})")?;
        }
        Command::PlaySe { audio } => {
            let audio_name = escape_string(&audio.name);

            write_indent(&mut writer, indent)?;
            writeln!(&mut writer, "play_se(")?;

            write_indent(&mut writer, indent + 1)?;
            writeln!(&mut writer, "audio=AudioFile(")?;

            write_indent(&mut writer, indent + 2)?;
            writeln!(&mut writer, "name='{audio_name}',")?;

            write_indent(&mut writer, indent + 2)?;
            writeln!(&mut writer, "pan={},", audio.pan)?;

            write_indent(&mut writer, indent + 2)?;
            writeln!(&mut writer, "pitch={},", audio.pitch)?;

            write_indent(&mut writer, indent + 2)?;
            writeln!(&mut writer, "volume={},", audio.volume)?;

            write_indent(&mut writer, indent + 1)?;
            writeln!(&mut writer, "),")?;

            write_indent(&mut writer, indent)?;
            writeln!(&mut writer, ")")?;
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
        Command::Script { lines } => {
            let data = lines.join("\\n");

            write_indent(&mut writer, indent)?;
            writeln!(&mut writer, "script(data=\'{data}\')")?;
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

fn write_indent<W>(mut writer: W, indent: u16) -> std::fmt::Result
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
