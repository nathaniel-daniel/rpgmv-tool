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
        command2py(config, *indent, command, &mut python)?;
    }

    Ok(python)
}

fn command2py(
    config: &Config,
    indent: u16,
    command: &Command,
    python: &mut String,
) -> anyhow::Result<()> {
    match command {
        Command::Nop => {}
        Command::ShowText {
            face_name,
            face_index,
            background,
            position_type,
            lines,
        } => {
            write_indent(python, indent);
            writeln!(python, "show_text(")?;

            write_indent(python, indent + 1);
            writeln!(python, "face_name='{face_name}',")?;

            write_indent(python, indent + 1);
            writeln!(python, "face_index={face_index},")?;

            write_indent(python, indent + 1);
            writeln!(python, "background={background},")?;

            write_indent(python, indent + 1);
            writeln!(python, "position_type={position_type},")?;

            write_indent(python, indent + 1);
            writeln!(python, "lines=[")?;

            for line in lines {
                let line = escape_string(line);

                write_indent(python, indent + 2);
                writeln!(python, "'{line}',")?;
            }

            write_indent(python, indent + 1);
            writeln!(python, "],")?;

            write_indent(python, indent);
            writeln!(python, ")")?;
        }
        Command::ShowChoices {
            choices,
            cancel_type,
            default_type,
            position_type,
            background,
        } => {
            write_indent(python, indent);
            writeln!(python, "show_choices(")?;

            write_indent(python, indent + 1);
            writeln!(python, "choices=[")?;

            for choice in choices {
                let choice = escape_string(choice);

                write_indent(python, indent + 2);
                writeln!(python, "'{choice}',")?;
            }

            write_indent(python, indent + 1);
            writeln!(python, "],")?;

            write_indent(python, indent + 1);
            writeln!(python, "cancel_type={cancel_type},")?;

            write_indent(python, indent + 1);
            writeln!(python, "default_type={default_type},")?;

            write_indent(python, indent + 1);
            writeln!(python, "position_type={position_type},")?;

            write_indent(python, indent + 1);
            writeln!(python, "background={background},")?;

            write_indent(python, indent);
            writeln!(python, ")")?;
        }
        Command::ConditionalBranch(command) => {
            write_indent(python, indent);
            write!(python, "if ")?;
            match command {
                ConditionalBranchCommand::Switch { id, check_true } => {
                    let name = config.get_switch_name(*id);
                    let check_true_str = if *check_true { "" } else { "not " };
                    writeln!(python, "{check_true_str}{name}:")?;
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

                    writeln!(python, "{lhs} {operation} {rhs}:")?;
                }
            }
        }
        Command::CommonEvent { id } => {
            let name = config.get_common_event_name(*id);

            write_indent(python, indent);
            writeln!(python, "{name}()")?;
        }
        Command::ControlSwitches {
            start_id,
            end_id,
            value,
        } => {
            for id in *start_id..(*end_id + 1) {
                let name = config.get_switch_name(id);
                let value = stringify_bool(*value);

                write_indent(python, indent);
                writeln!(python, "{name} = {value}")?;
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
                write_indent(python, indent);
                writeln!(python, "{name} {operation} {value}")?;
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

            write_indent(python, indent);
            writeln!(python, "gain_item(item={item}, value={sign}{value})")?;
        }
        Command::ChangeSaveAccess { disable } => {
            let fn_name = if *disable {
                "disable_saving"
            } else {
                "enable_saving"
            };
            write_indent(python, indent);
            writeln!(python, "{fn_name}()")?
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
            write_indent(python, indent);
            writeln!(python, "transfer_player({map_arg}, x={x}, y={y}, direction={direction}, fade_type={fade_type})")?;
        }
        Command::SetMovementRoute {
            character_id,
            route,
        } => {
            let repeat = stringify_bool(route.repeat);
            let skippable = stringify_bool(route.skippable);
            let wait = stringify_bool(route.wait);

            write_indent(python, indent);
            writeln!(python, "set_movement_route(")?;

            write_indent(python, indent + 1);
            writeln!(python, "character_id={character_id},")?;

            write_indent(python, indent + 1);
            writeln!(python, "route=MoveRoute(")?;

            write_indent(python, indent + 2);
            writeln!(python, "repeat={repeat},")?;

            write_indent(python, indent + 2);
            writeln!(python, "skippable={skippable},")?;

            write_indent(python, indent + 2);
            writeln!(python, "wait={wait},")?;

            write_indent(python, indent + 2);
            writeln!(python, "list=[")?;

            for command in route.list.iter() {
                let command_indent = command
                    .indent
                    .map(|indent| indent.to_string())
                    .unwrap_or_else(|| "None".to_string());

                write_indent(python, indent + 3);
                writeln!(python, "MoveCommand(")?;

                write_indent(python, indent + 4);
                writeln!(python, "code={},", command.code)?;

                write_indent(python, indent + 4);
                writeln!(python, "indent={command_indent},")?;

                match command.parameters.as_ref() {
                    Some(parameters) => {
                        write_indent(python, indent + 4);
                        writeln!(python, "parameters=[")?;

                        for parameter in parameters {
                            write_indent(python, indent + 5);

                            match parameter {
                                serde_json::Value::Number(number) if number.is_i64() => {
                                    let value = number.as_i64().context("value is not an i64")?;
                                    writeln!(python, "{value},")?;
                                }
                                _ => bail!("cannot write parameter \"{parameter:?}\""),
                            }
                        }

                        write_indent(python, indent + 4);
                        writeln!(python, "],")?;
                    }
                    None => {
                        write_indent(python, indent + 4);
                        writeln!(python, "parameters=None,")?;
                    }
                }

                write_indent(python, indent + 3);
                writeln!(python, "),")?;
            }

            write_indent(python, indent + 2);
            writeln!(python, "]")?;

            write_indent(python, indent + 1);
            writeln!(python, "),")?;

            write_indent(python, indent);
            writeln!(python, ")")?;
        }
        Command::ChangeTransparency { set_transparent } => {
            let set_transparent = stringify_bool(*set_transparent);

            write_indent(python, indent);
            writeln!(
                python,
                "change_transparency(set_transparent={set_transparent})"
            )?
        }
        Command::ShowBalloonIcon {
            character_id,
            balloon_id,
            wait,
        } => {
            let wait = stringify_bool(*wait);

            write_indent(python, indent);
            writeln!(python, "show_balloon_icon(character_id={character_id}, balloon_id={balloon_id}, wait={wait})")?
        }
        Command::FadeoutScreen => {
            write_indent(python, indent);
            writeln!(python, "fadeout_screen()")?
        }
        Command::FadeinScreen => {
            write_indent(python, indent);
            writeln!(python, "fadein_screen()")?
        }
        Command::TintScreen {
            tone,
            duration,
            wait,
        } => {
            let wait = stringify_bool(*wait);

            write_indent(python, indent);
            writeln!(
                python,
                "tint_screen(tone={tone:?}, duration={duration}, wait={wait})"
            )?
        }
        Command::FlashScreen {
            color,
            duration,
            wait,
        } => {
            let wait = stringify_bool(*wait);

            write_indent(python, indent);
            writeln!(
                python,
                "flash_screen(color={color:?}, duration={duration}, wait={wait})"
            )?
        }
        Command::Wait { duration } => {
            write_indent(python, indent);
            writeln!(python, "wait(duration={duration})")?
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

            write_indent(python, indent);
            writeln!(python, "show_picture(")?;

            write_indent(python, indent + 1);
            writeln!(python, "picture_id={picture_id},")?;

            write_indent(python, indent + 1);
            writeln!(python, "picture_name='{picture_name}',")?;

            write_indent(python, indent + 1);
            writeln!(python, "origin={origin},")?;

            write_indent(python, indent + 1);
            writeln!(python, "x={x},")?;

            write_indent(python, indent + 1);
            writeln!(python, "y={y},")?;

            write_indent(python, indent + 1);
            writeln!(python, "scale_x={scale_x},")?;

            write_indent(python, indent + 1);
            writeln!(python, "scale_y={scale_y},")?;

            write_indent(python, indent + 1);
            writeln!(python, "opacity={opacity},")?;

            write_indent(python, indent + 1);
            writeln!(python, "blend_mode={blend_mode},")?;

            write_indent(python, indent);
            writeln!(python, ")")?;
        }
        Command::ErasePicture { picture_id } => {
            write_indent(python, indent);
            writeln!(python, "erase_picture(picture_id={picture_id})")?;
        }
        Command::PlaySe { audio } => {
            let audio_name = escape_string(&audio.name);

            write_indent(python, indent);
            writeln!(python, "play_se(")?;

            write_indent(python, indent + 1);
            writeln!(python, "audio=AudioFile(")?;

            write_indent(python, indent + 2);
            writeln!(python, "name='{audio_name}',")?;

            write_indent(python, indent + 2);
            writeln!(python, "pan={},", audio.pan)?;

            write_indent(python, indent + 2);
            writeln!(python, "pitch={},", audio.pitch)?;

            write_indent(python, indent + 2);
            writeln!(python, "volume={},", audio.volume)?;

            write_indent(python, indent + 1);
            writeln!(python, "),")?;

            write_indent(python, indent);
            writeln!(python, ")")?;
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

            write_indent(python, indent);
            writeln!(python, "{fn_name}({actor_arg}, skill={skill})")?;
        }
        Command::When {
            choice_index,
            choice_name,
        } => {
            write_indent(python, indent);
            writeln!(
                python,
                "if get_choice_index() == {choice_index}: # {choice_name}"
            )?;
        }
        Command::WhenEnd => {
            // Trust indents over end commands
        }
        Command::Else => {
            write_indent(python, indent);
            writeln!(python, "else:")?;
        }
        Command::ConditionalBranchEnd => {
            // Trust indents over end commands
        }
        Command::Unknown { code, parameters } => {
            write_indent(python, indent);
            writeln!(
                python,
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

fn write_indent(string: &mut String, indent: u16) {
    for _ in 0..indent {
        string.push('\t');
    }
}

fn escape_string(input: &str) -> String {
    input.replace('\'', "\\'")
}
