use crate::is_game_mv;
use crate::message_parser::MessageNode;
use crate::message_parser::MessageParser;
use crate::parse_map_name;
use crate::util::Font;
use crate::util::get_text_width;
use anyhow::Context;
use regex::Regex;
use rpgmv_types::Plugin;
use rpgmv_types::System;
use std::collections::VecDeque;
use std::path::Path;
use std::path::PathBuf;
use std::sync::LazyLock;

/// Message text is padded by default with 18px.
const MESSAGE_STANDARD_PADDING: u16 = 18;

/// Message Actor Faces are 144x144 px.
const MESSAGE_FACE_SIZE: u16 = 144;
const MESSAGE_FACE_PADDING: u16 = 12;

/// MV only
fn load_plugins_js(game_path: &Path) -> anyhow::Result<Vec<Plugin>> {
    static REGEX: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"(?s)var \$plugins =\s*(\[.*\]);").unwrap());

    // TODO: If we ever add MZ support, change the path here conditonally.
    let js_dir = game_path.join("www").join("js");
    let plugins_js_path = js_dir.join("plugins.js");
    let plugins_js_raw = std::fs::read_to_string(plugins_js_path)?;
    let plugins_js_string = REGEX
        .captures(&plugins_js_raw)
        .context("failed to locate plugins with regex")?
        .get(1)
        .context("missing group")?
        .as_str();
    let plugins = serde_json::from_str(plugins_js_string)?;
    Ok(plugins)
}

pub struct CheckLineSizeEntry {
    pub file: String,
    pub line: String,
    pub text_width: f32,
    pub target_width: u16,
    pub suggested_line: String,
}

struct CheckLineSizeContext {
    font: Font,
    font_size: u16,
    game_width: u16,

    entries: VecDeque<CheckLineSizeEntry>,
}

impl CheckLineSizeContext {
    fn new(font: Font, font_size: u16, game_width: u16) -> Self {
        Self {
            font,
            font_size,
            game_width,
            entries: VecDeque::new(),
        }
    }

    #[expect(unused)]
    fn pop_entry(&mut self) -> Option<CheckLineSizeEntry> {
        self.entries.pop_front()
    }

    fn get_text_width(&mut self, text: &str) -> anyhow::Result<f32> {
        get_text_width(&mut self.font, text, Some(self.font_size.into()))
            .context("failed to get text width")
    }

    fn suggest_line_replacement(
        &mut self,
        line: &str,
        target_width: u16,
    ) -> anyhow::Result<Option<String>> {
        if self.get_text_width(line)? < f32::from(target_width) {
            return Ok(Some(line.to_string()));
        }

        // This can be a lot more efficient if we use raw harfbuzz/whatever text layout library we are using.
        let mut words = line.split(' ').collect::<Vec<_>>();
        while !words.is_empty() {
            words.pop();
            let candidate = words.join(" ");
            if candidate.is_empty() {
                return Ok(None);
            }
            let suggested_width = self.get_text_width(&candidate)?;
            if suggested_width < f32::from(target_width) {
                return Ok(Some(candidate));
            }
        }

        Ok(None)
    }

    fn check_event_command_message(
        &mut self,
        face: &str,
        lines: &str,
        file: &str,
    ) -> anyhow::Result<()> {
        // Strip escape sequences.
        let mut parser = MessageParser::new(lines);
        let nodes = parser
            .parse()
            .with_context(|| format!("failed to parse \"{lines}\""))?;
        let mut stripped_lines = String::with_capacity(lines.len());
        for node in nodes {
            match node {
                MessageNode::Text { value } => stripped_lines.push_str(&value),
                MessageNode::TextCode { .. } | MessageNode::TextCodeWithBody { .. } => {
                    // Ignore text codes
                }
            }
        }

        for line in stripped_lines.split('\n') {
            if line.is_empty() {
                continue;
            }

            let text_width = self.get_text_width(line)?;
            let mut target_width = self.game_width - (2 * MESSAGE_STANDARD_PADDING);
            if !face.is_empty() {
                target_width -= MESSAGE_FACE_SIZE + (MESSAGE_FACE_PADDING * 2);
            }

            if text_width >= f32::from(target_width) {
                let suggested_line = self.suggest_line_replacement(line, target_width)?;
                let suggested_line = suggested_line.as_deref().unwrap_or("None");

                self.entries.push_back(CheckLineSizeEntry {
                    file: file.to_string(),
                    line: line.to_string(),
                    text_width,
                    target_width,
                    suggested_line: suggested_line.to_string(),
                });
            }
        }

        Ok(())
    }

    fn check_event_command_list(
        &mut self,
        command_list: &[rpgmv_types::EventCommand],
        file: &str,
    ) -> anyhow::Result<()> {
        let mut last_face = "";
        let mut lines = None;
        for command in command_list.iter() {
            let code = command.code;
            let parameters = &command.parameters;

            match code {
                101 => {
                    if let Some(lines) = lines.as_deref() {
                        self.check_event_command_message(last_face, lines, file)?;
                    }

                    last_face = parameters
                        .first()
                        .context("missing last face")?
                        .as_str()
                        .context("face is not a string")?;
                    lines = Some(String::new());
                }
                401 => {
                    let line = parameters
                        .first()
                        .context("missing line")?
                        .as_str()
                        .context("line is not a string")?;

                    let lines = lines
                        .as_mut()
                        .context("encountered 401 command out of message")?;
                    lines.push('\n');
                    lines.push_str(line);
                }
                _ => {
                    if let Some(lines) = lines.as_deref() {
                        self.check_event_command_message(last_face, lines, file)?;
                    }

                    last_face = "";
                    lines = None;
                }
            }
        }

        if let Some(lines) = lines.as_deref() {
            self.check_event_command_message(last_face, lines, file)?;
        }

        Ok(())
    }

    fn check_map(&mut self, map: &rpgmv_types::Map, map_number: u16) -> anyhow::Result<()> {
        let file = format!("Map{map_number:03}");
        for event in map.events.iter() {
            let event = match event {
                Some(event) => event,
                None => continue,
            };

            for page in event.pages.iter() {
                self.check_event_command_list(&page.list, &file)?;
            }
        }

        Ok(())
    }

    fn check_common_events(
        &mut self,
        common_events: &[Option<rpgmv_types::CommonEvent>],
    ) -> anyhow::Result<()> {
        let file = "CommonEvents";
        for common_event in common_events.iter() {
            let common_event = match common_event {
                Some(common_event) => common_event,
                None => continue,
            };

            self.check_event_command_list(&common_event.list, file)?;
        }
        Ok(())
    }
}

/// Check lines for text overflow in a game.
pub fn check_line_size(game_path: &Path) -> anyhow::Result<Vec<CheckLineSizeEntry>> {
    let game_is_mv = is_game_mv(game_path)?;

    let (font_name, font_size, game_width) = if game_is_mv {
        let plugins = load_plugins_js(game_path)?;

        let mut screen_width = None;
        for plugin in plugins {
            if plugin.name == "Community_Basic" {
                let screen_width_param = plugin
                    .parameters
                    .get("screenWidth")
                    .context("missing screen width param")?
                    .parse()
                    .context("screen width is not a u16")?;
                screen_width = Some(screen_width_param);
            }
        }

        (
            "mplus-1m-regular.ttf".to_string(),
            28,
            screen_width.unwrap_or(816),
        )
    } else {
        let system_path = game_path.join("data").join("System.json");
        let system_string = std::fs::read_to_string(&system_path)
            .with_context(|| format!("failed to read to string \"{}\"", system_path.display()))?;
        let system: System =
            serde_json::from_str(&system_string).context("failed to parse system json")?;
        let system_advanced = system
            .advanced
            .context("System missing \"advanced\" field")?;

        (
            system_advanced.main_font_filename,
            system_advanced.font_size,
            system_advanced.screen_width,
        )
    };

    let font_path = {
        let mut font_path = PathBuf::from(game_path);
        if game_is_mv {
            font_path.push("www");
        }
        font_path.push("fonts");
        font_path.push(font_name);
        font_path
    };
    let font = crate::util::load_font(&font_path)?;

    let mut context = CheckLineSizeContext::new(font, font_size, game_width);

    let data_path = {
        let mut path = PathBuf::from(game_path);
        if game_is_mv {
            path.push("www");
        }
        path.push("data");
        path
    };
    let mut dir_entries = std::fs::read_dir(data_path)?.collect::<Result<Vec<_>, _>>()?;
    dir_entries.sort_by_key(|entry_a| entry_a.file_name());
    for entry in dir_entries {
        let entry_file_name = entry
            .file_name()
            .to_str()
            .context("file name is not unicode")?
            .to_string();
        let entry_path = entry.path();

        if !entry_file_name.ends_with(".json") {
            continue;
        }
        if entry_file_name == "MapInfos.json" {
            continue;
        }

        if let Some(map_number) = parse_map_name(&entry_file_name) {
            let string = std::fs::read_to_string(&entry_path)
                .with_context(|| format!("failed to read \"{}\"", entry_path.display()))?;

            let map: rpgmv_types::Map = serde_json::from_str(&string)?;
            context.check_map(&map, map_number)?;
        } else if entry_file_name == "CommonEvents.json" {
            let string = std::fs::read_to_string(&entry_path)
                .with_context(|| format!("failed to read \"{}\"", entry_path.display()))?;
            let value: Vec<Option<rpgmv_types::CommonEvent>> = serde_json::from_str(&string)?;
            context.check_common_events(&value)?;
        }
    }

    Ok(context.entries.into_iter().collect())
}
