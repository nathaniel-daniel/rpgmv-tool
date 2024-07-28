use super::Command;
use super::ConditionalBranchCommand;
use super::Config;
use super::ControlVariablesValue;
use super::MaybeRef;
use anyhow::bail;
use anyhow::Context;
use std::fmt::Write;

pub fn commands2py(config: &Config, commands: &[(u16, Command)]) -> anyhow::Result<String> {
    let mut python = String::new();
    for (indent, command) in commands.iter() {
        match command {
            Command::Nop => {}
            Command::ShowText {
                face_name,
                face_index,
                background,
                position_type,
                lines,
            } => {
                write_indent(&mut python, *indent);
                writeln!(&mut python, "show_text(")?;

                write_indent(&mut python, *indent + 1);
                writeln!(&mut python, "face_name='{face_name}',")?;

                write_indent(&mut python, *indent + 1);
                writeln!(&mut python, "face_index={face_index},")?;

                write_indent(&mut python, *indent + 1);
                writeln!(&mut python, "background={background},")?;

                write_indent(&mut python, *indent + 1);
                writeln!(&mut python, "position_type={position_type},")?;

                write_indent(&mut python, *indent + 1);
                writeln!(&mut python, "lines=[")?;

                for line in lines {
                    let line = escape_string(line);

                    write_indent(&mut python, *indent + 2);
                    writeln!(&mut python, "'{line}',")?;
                }

                write_indent(&mut python, *indent + 1);
                writeln!(&mut python, "],")?;

                write_indent(&mut python, *indent);
                writeln!(&mut python, ")")?;
            }
            Command::ShowChoices {
                choices,
                cancel_type,
                default_type,
                position_type,
                background,
            } => {
                write_indent(&mut python, *indent);
                writeln!(&mut python, "show_choices(")?;

                write_indent(&mut python, *indent + 1);
                writeln!(&mut python, "choices=[")?;

                for choice in choices {
                    let choice = escape_string(choice);

                    write_indent(&mut python, *indent + 2);
                    writeln!(&mut python, "'{choice}',")?;
                }

                write_indent(&mut python, *indent + 1);
                writeln!(&mut python, "],")?;

                write_indent(&mut python, *indent + 1);
                writeln!(&mut python, "cancel_type={cancel_type},")?;

                write_indent(&mut python, *indent + 1);
                writeln!(&mut python, "default_type={default_type},")?;

                write_indent(&mut python, *indent + 1);
                writeln!(&mut python, "position_type={position_type},")?;

                write_indent(&mut python, *indent + 1);
                writeln!(&mut python, "background={background},")?;

                write_indent(&mut python, *indent);
                writeln!(&mut python, ")")?;
            }
            Command::ConditionalBranch(command) => {
                write_indent(&mut python, *indent);
                write!(&mut python, "if ")?;
                match command {
                    ConditionalBranchCommand::Switch { id, check_true } => {
                        let name = config.get_switch_name(*id);
                        let check_true_str = if *check_true { "" } else { "not " };
                        writeln!(&mut python, "{check_true_str}{name}:")?;
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

                        writeln!(&mut python, "{lhs} {operation} {rhs}:")?;
                    }
                }
            }
            Command::CommonEvent { id } => {
                let name = config.get_common_event_name(*id);

                write_indent(&mut python, *indent);
                writeln!(&mut python, "{name}()")?;
            }
            Command::ControlSwitches {
                start_id,
                end_id,
                value,
            } => {
                for id in *start_id..(*end_id + 1) {
                    let name = config.get_switch_name(id);
                    let value = stringify_bool(*value);

                    write_indent(&mut python, *indent);
                    writeln!(&mut python, "{name} = {value}")?;
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
                };
                for variable_id in *start_variable_id..(*end_variable_id + 1) {
                    let name = config.get_variable_name(variable_id);
                    write_indent(&mut python, *indent);
                    writeln!(&mut python, "{name} {operation} {value}")?;
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

                write_indent(&mut python, *indent);
                writeln!(&mut python, "gain_item(item={item}, value={sign}{value})")?;
            }
            Command::ChangeSaveAccess { disable } => {
                let fn_name = if *disable {
                    "disable_saving"
                } else {
                    "enable_saving"
                };
                write_indent(&mut python, *indent);
                writeln!(&mut python, "{fn_name}()")?
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
                write_indent(&mut python, *indent);
                writeln!(&mut python, "transfer_player({map_arg}, x={x}, y={y}, direction={direction}, fade_type={fade_type})")?;
            }
            Command::SetMovementRoute {
                character_id,
                route,
            } => {
                let repeat = stringify_bool(route.repeat);
                let skippable = stringify_bool(route.skippable);
                let wait = stringify_bool(route.wait);

                write_indent(&mut python, *indent);
                writeln!(&mut python, "set_movement_route(")?;

                write_indent(&mut python, *indent + 1);
                writeln!(&mut python, "character_id={character_id},")?;

                write_indent(&mut python, *indent + 1);
                writeln!(&mut python, "route=MoveRoute(")?;

                write_indent(&mut python, *indent + 2);
                writeln!(&mut python, "repeat={repeat},")?;

                write_indent(&mut python, *indent + 2);
                writeln!(&mut python, "skippable={skippable},")?;

                write_indent(&mut python, *indent + 2);
                writeln!(&mut python, "wait={wait},")?;

                write_indent(&mut python, *indent + 2);
                writeln!(&mut python, "list=[")?;

                for command in route.list.iter() {
                    let command_indent = command
                        .indent
                        .map(|indent| indent.to_string())
                        .unwrap_or_else(|| "None".to_string());

                    write_indent(&mut python, *indent + 3);
                    writeln!(&mut python, "MoveCommand(")?;

                    write_indent(&mut python, *indent + 4);
                    writeln!(&mut python, "code={},", command.code)?;

                    write_indent(&mut python, *indent + 4);
                    writeln!(&mut python, "indent={command_indent},")?;

                    match command.parameters.as_ref() {
                        Some(parameters) => {
                            write_indent(&mut python, *indent + 4);
                            writeln!(&mut python, "parameters=[")?;

                            for parameter in parameters {
                                write_indent(&mut python, *indent + 5);

                                match parameter {
                                    serde_json::Value::Number(number) if number.is_i64() => {
                                        let value =
                                            number.as_i64().context("value is not an i64")?;
                                        writeln!(&mut python, "{value},")?;
                                    }
                                    _ => bail!("cannot write parameter \"{parameter:?}\""),
                                }
                            }

                            write_indent(&mut python, *indent + 4);
                            writeln!(&mut python, "],")?;
                        }
                        None => {
                            write_indent(&mut python, *indent + 4);
                            writeln!(&mut python, "parameters=None,")?;
                        }
                    }

                    write_indent(&mut python, *indent + 3);
                    writeln!(&mut python, "),")?;
                }

                write_indent(&mut python, *indent + 2);
                writeln!(&mut python, "]")?;

                write_indent(&mut python, *indent + 1);
                writeln!(&mut python, "),")?;

                write_indent(&mut python, *indent);
                writeln!(&mut python, ")")?;
            }
            Command::ChangeTransparency { set_transparent } => {
                let set_transparent = stringify_bool(*set_transparent);

                write_indent(&mut python, *indent);
                writeln!(
                    &mut python,
                    "change_transparency(set_transparent={set_transparent})"
                )?
            }
            Command::ShowBalloonIcon {
                character_id,
                balloon_id,
                wait,
            } => {
                let wait = stringify_bool(*wait);

                write_indent(&mut python, *indent);
                writeln!(&mut python, "show_balloon_icon(character_id={character_id}, balloon_id={balloon_id}, wait={wait})")?
            }
            Command::FadeoutScreen => {
                write_indent(&mut python, *indent);
                writeln!(&mut python, "fadeout_screen()")?
            }
            Command::FadeinScreen => {
                write_indent(&mut python, *indent);
                writeln!(&mut python, "fadein_screen()")?
            }
            Command::TintScreen {
                tone,
                duration,
                wait,
            } => {
                let wait = stringify_bool(*wait);

                write_indent(&mut python, *indent);
                writeln!(
                    &mut python,
                    "tint_screen(tone={tone:?}, duration={duration}, wait={wait})"
                )?
            }
            Command::FlashScreen {
                color,
                duration,
                wait,
            } => {
                let wait = stringify_bool(*wait);

                write_indent(&mut python, *indent);
                writeln!(
                    &mut python,
                    "flash_screen(color={color:?}, duration={duration}, wait={wait})"
                )?
            }
            Command::Wait { duration } => {
                write_indent(&mut python, *indent);
                writeln!(&mut python, "wait(duration={duration})")?
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

                write_indent(&mut python, *indent);
                writeln!(&mut python, "show_picture(")?;

                write_indent(&mut python, *indent + 1);
                writeln!(&mut python, "picture_id={picture_id},")?;

                write_indent(&mut python, *indent + 1);
                writeln!(&mut python, "picture_name='{picture_name}',")?;

                write_indent(&mut python, *indent + 1);
                writeln!(&mut python, "origin={origin},")?;

                write_indent(&mut python, *indent + 1);
                writeln!(&mut python, "x={x},")?;

                write_indent(&mut python, *indent + 1);
                writeln!(&mut python, "y={y},")?;

                write_indent(&mut python, *indent + 1);
                writeln!(&mut python, "scale_x={scale_x},")?;

                write_indent(&mut python, *indent + 1);
                writeln!(&mut python, "scale_y={scale_y},")?;

                write_indent(&mut python, *indent + 1);
                writeln!(&mut python, "opacity={opacity},")?;

                write_indent(&mut python, *indent + 1);
                writeln!(&mut python, "blend_mode={blend_mode},")?;

                write_indent(&mut python, *indent);
                writeln!(&mut python, ")")?;
            }
            Command::ErasePicture { picture_id } => {
                write_indent(&mut python, *indent);
                writeln!(&mut python, "erase_picture(picture_id={picture_id})")?;
            }
            Command::PlaySe { audio } => {
                let audio_name = escape_string(&audio.name);

                write_indent(&mut python, *indent);
                writeln!(&mut python, "play_se(")?;

                write_indent(&mut python, *indent + 1);
                writeln!(&mut python, "audio=AudioFile(")?;

                write_indent(&mut python, *indent + 2);
                writeln!(&mut python, "name='{audio_name}',")?;

                write_indent(&mut python, *indent + 2);
                writeln!(&mut python, "pan={},", audio.pan)?;

                write_indent(&mut python, *indent + 2);
                writeln!(&mut python, "pitch={},", audio.pitch)?;

                write_indent(&mut python, *indent + 2);
                writeln!(&mut python, "volume={},", audio.volume)?;

                write_indent(&mut python, *indent + 1);
                writeln!(&mut python, "),")?;

                write_indent(&mut python, *indent);
                writeln!(&mut python, ")")?;
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

                write_indent(&mut python, *indent);
                writeln!(&mut python, "{fn_name}({actor_arg}, skill={skill})")?;
            }
            Command::When {
                choice_index,
                choice_name,
            } => {
                write_indent(&mut python, *indent);
                writeln!(
                    &mut python,
                    "if get_choice_index() == {choice_index}: # {choice_name}"
                )?;
            }
            Command::WhenEnd => {
                // Trust indents over end commands
            }
            Command::Else => {
                write_indent(&mut python, *indent);
                writeln!(&mut python, "else:")?;
            }
            Command::ConditionalBranchEnd => {
                // Trust indents over end commands
            }
            Command::Unknown { code, parameters } => {
                write_indent(&mut python, *indent);
                writeln!(
                    &mut python,
                    "# Unknown Command Code {code:?}, parameters: {parameters:?}"
                )?;
            }
        }
    }

    Ok(python)
}

fn stringify_bool(b: bool) -> &'static str {
    match b {
        true => "True",
        false => "False",
    }
}

fn write_indent(string: &mut String, indent: u16) {
    for _ in 0..indent {
        string.push('\t');
    }
}

fn escape_string(input: &str) -> String {
    input.replace('\'', "\\'")
}
