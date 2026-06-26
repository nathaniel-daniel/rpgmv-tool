use crate::Font;
use crate::get_text_width;
use crate::is_game_mv;
use crate::load_font;
use crate::message_parser::MessageNode;
use crate::message_parser::MessageParser;
use crate::parse_map_name;
use anyhow::Context;
use regex::Regex;
use rpgmv_types::Plugin;
use rpgmv_types::System;
use std::collections::HashSet;
use std::collections::VecDeque;
use std::path::Path;
use std::path::PathBuf;
use std::sync::LazyLock;

/// Message text is padded by default with 18px.
const MESSAGE_STANDARD_PADDING: u16 = 18;

/// Message Actor Faces are 144x144 px.
const MESSAGE_FACE_SIZE: u16 = 144;
const MESSAGE_FACE_PADDING: u16 = 12;
/// Text padding is applied in addition to STANDARD_PADDING, but not for all windows.
const TEXT_PADDING: u16 = 6;

fn read_to_string<P>(path: P) -> anyhow::Result<String>
where
    P: AsRef<Path>,
{
    let path = path.as_ref();
    std::fs::read_to_string(path).with_context(|| format!("failed to read \"{}\"", path.display()))
}

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
    extra_single_text_codes: HashSet<char>,
    extra_text_codes: HashSet<String>,

    entries: VecDeque<CheckLineSizeEntry>,
}

impl CheckLineSizeContext {
    fn new(
        font: Font,
        font_size: u16,
        game_width: u16,
        extra_single_text_codes: HashSet<char>,
        extra_text_codes: HashSet<String>,
    ) -> Self {
        Self {
            font,
            font_size,
            game_width,
            extra_single_text_codes,
            extra_text_codes,

            entries: VecDeque::new(),
        }
    }

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
        for single_text_code in self.extra_single_text_codes.iter() {
            parser.add_single_text_code(*single_text_code);
        }
        for text_code in self.extra_text_codes.iter() {
            parser.add_text_code(text_code);
        }
        let nodes = parser
            .parse()
            .with_context(|| format!("failed to parse \"{lines}\""))?;
        let mut stripped_lines = String::with_capacity(lines.len());
        for node in nodes {
            match node {
                MessageNode::Text { value } => stripped_lines.push_str(&value),
                MessageNode::TextCode { .. }
                | MessageNode::TextCodeWithBody { .. }
                | MessageNode::YepTextCodeWithBody { .. } => {
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

    fn check_item_description(&mut self, description: &str, file: &str) -> anyhow::Result<()> {
        for line in description.split('\n') {
            if line.is_empty() {
                continue;
            }

            let text_width = self.get_text_width(line)?;
            let target_width = self.game_width - (2 * (MESSAGE_STANDARD_PADDING + TEXT_PADDING));

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

    fn check_troops(&mut self, troops: &[Option<rpgmv_types::Troop>]) -> anyhow::Result<()> {
        let file = "Troops";
        for troop in troops.iter() {
            let troop = match troop {
                Some(troop) => troop,
                None => continue,
            };

            for page in troop.pages.iter() {
                self.check_event_command_list(&page.list, file)?;
            }
        }
        Ok(())
    }

    fn check_armors(&mut self, armors: &[Option<rpgmv_types::Armor>]) -> anyhow::Result<()> {
        let file = "Armors";
        for armor in armors.iter() {
            let armor = match armor {
                Some(armor) => armor,
                None => continue,
            };

            self.check_item_description(&armor.description, file)?;
        }
        Ok(())
    }

    fn check_items(&mut self, items: &[Option<rpgmv_types::Item>]) -> anyhow::Result<()> {
        let file = "Items";
        for item in items.iter() {
            let item = match item {
                Some(item) => item,
                None => continue,
            };

            self.check_item_description(&item.description, file)?;
        }
        Ok(())
    }

    fn check_skills(&mut self, skills: &[Option<rpgmv_types::Skill>]) -> anyhow::Result<()> {
        let file = "Skills";
        for skill in skills.iter() {
            let skill = match skill {
                Some(skill) => skill,
                None => continue,
            };

            self.check_item_description(&skill.description, file)?;
        }
        Ok(())
    }

    fn check_weapons(&mut self, weapons: &[Option<rpgmv_types::Weapon>]) -> anyhow::Result<()> {
        let file = "Weapons";
        for weapon in weapons.iter() {
            let weapon = match weapon {
                Some(weapon) => weapon,
                None => continue,
            };

            self.check_item_description(&weapon.description, file)?;
        }
        Ok(())
    }
}

pub struct CheckLineSizeIter {
    found_error: bool,
    dir_entries: std::vec::IntoIter<std::fs::DirEntry>,
    context: CheckLineSizeContext,
}

impl CheckLineSizeIter {
    fn new(path: &Path, context: CheckLineSizeContext) -> anyhow::Result<Self> {
        let mut dir_entries = std::fs::read_dir(path)?.collect::<Result<Vec<_>, _>>()?;
        dir_entries.sort_by_key(|entry_a| entry_a.file_name());

        Ok(Self {
            found_error: false,
            dir_entries: dir_entries.into_iter(),
            context,
        })
    }
}

impl Iterator for CheckLineSizeIter {
    type Item = anyhow::Result<CheckLineSizeEntry>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.found_error {
            return None;
        }

        if let Some(entry) = self.context.pop_entry() {
            return Some(Ok(entry));
        }

        loop {
            let entry = self.dir_entries.next()?;

            let result = (|| {
                let entry_file_name = entry
                    .file_name()
                    .to_str()
                    .context("file name is not unicode")?
                    .to_string();

                let entry_path = entry.path();

                if !entry_file_name.ends_with(".json") {
                    return Ok(());
                }
                if entry_file_name == "MapInfos.json" {
                    return Ok(());
                }

                if let Some(map_number) = parse_map_name(&entry_file_name) {
                    let string = read_to_string(&entry_path)?;
                    let map: rpgmv_types::Map = serde_json::from_str(&string)
                        .with_context(|| format!("failed to parse Map {map_number}"))?;
                    self.context.check_map(&map, map_number)?;
                } else {
                    match entry_file_name.as_str() {
                        "CommonEvents.json" => {
                            let string = read_to_string(&entry_path)?;
                            let value: Vec<Option<rpgmv_types::CommonEvent>> =
                                serde_json::from_str(&string)
                                    .context("failed to parse CommonEvents")?;
                            self.context.check_common_events(&value)?;
                        }
                        "Troops.json" => {
                            let string = read_to_string(&entry_path)?;
                            let value: Vec<Option<rpgmv_types::Troop>> =
                                serde_json::from_str(&string).context("failed to parse Troops")?;
                            self.context.check_troops(&value)?;
                        }
                        "Armors.json" => {
                            let string = read_to_string(&entry_path)?;
                            let value: Vec<Option<rpgmv_types::Armor>> =
                                serde_json::from_str(&string).context("failed to parse Armors")?;
                            self.context.check_armors(&value)?;
                        }
                        "Items.json" => {
                            let string = read_to_string(&entry_path)?;
                            let value: Vec<Option<rpgmv_types::Item>> =
                                serde_json::from_str(&string).context("failed to parse Items")?;
                            self.context.check_items(&value)?;
                        }
                        "Skills.json" => {
                            let string = read_to_string(&entry_path)?;
                            let value: Vec<Option<rpgmv_types::Skill>> =
                                serde_json::from_str(&string).context("failed to parse Skills")?;
                            self.context.check_skills(&value)?;
                        }
                        "Weapons.json" => {
                            let string = read_to_string(&entry_path)?;
                            let value: Vec<Option<rpgmv_types::Weapon>> =
                                serde_json::from_str(&string).context("failed to parse Weapons")?;
                            self.context.check_weapons(&value)?;
                        }
                        _ => {}
                    }
                }

                anyhow::Ok(())
            })();

            match result {
                Ok(()) => {
                    if let Some(entry) = self.context.pop_entry() {
                        return Some(Ok(entry));
                    }
                }
                Err(error) => {
                    self.found_error = true;
                    return Some(Err(error));
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct CheckLineSizeOptions {
    /// The game path.
    ///
    /// Used for loading fonts.
    pub game_path: PathBuf,

    /// Whether the game is mv.
    pub game_is_mv: bool,

    /// The font name
    pub font_name: String,

    /// The font size
    pub font_size: u16,

    /// The screen width
    pub screen_width: u16,

    /// The data path
    pub data_path: PathBuf,

    /// Extra single text codes
    ///
    /// RPGMaker games can use plugins that extend the standard single text code set.
    pub extra_single_text_codes: HashSet<char>,

    /// Extra text codes
    ///
    /// RPGMaker games can use plugins that extend the standard text code set.
    pub extra_text_codes: HashSet<String>,
}

impl CheckLineSizeOptions {
    /// Create options for checking a game from a game path.
    pub fn from_game_path<P>(game_path: P) -> anyhow::Result<Self>
    where
        P: AsRef<Path>,
    {
        let game_path = game_path.as_ref();

        let game_is_mv = is_game_mv(game_path)?;

        let (font_name, font_size, screen_width) = if game_is_mv {
            let plugins = load_plugins_js(game_path)?;

            let mut screen_width: Option<u16> = None;
            for plugin in plugins {
                match plugin.name.as_str() {
                    "Community_Basic" => {
                        let screen_width_param = plugin
                            .parameters
                            .get("screenWidth")
                            .context("Missing Community_Basic screenWidth")?;
                        let screen_width_param = if screen_width_param.is_empty() {
                            None
                        } else {
                            Some(
                                screen_width_param
                                    .parse()
                                    .context("Community_Basic screenWidth is not a u16")?,
                            )
                        };
                        // This plugin will set a default if not present.
                        let screen_width_param = screen_width_param.unwrap_or(816);
                        screen_width = Some(screen_width_param);
                    }
                    "YEP_CoreEngine" => {
                        let screen_width_param = plugin
                            .parameters
                            .get("Screen Width")
                            .context("Missing YEP_CoreEngine Screen Width")?;
                        let screen_width_param = if screen_width_param.is_empty() {
                            None
                        } else {
                            Some(
                                screen_width_param
                                    .parse()
                                    .context("YEP_CoreEngine Screen Width is not a u16")?,
                            )
                        };
                        // This plugin will set a default if not present.
                        let screen_width_param = screen_width_param.unwrap_or(816);
                        screen_width = Some(screen_width_param);
                    }
                    _ => {}
                }
            }

            (
                "mplus-1m-regular.ttf".to_string(),
                28,
                screen_width.unwrap_or(816),
            )
        } else {
            let system_path = game_path.join("data").join("System.json");
            let system_string = std::fs::read_to_string(&system_path).with_context(|| {
                format!("failed to read to string \"{}\"", system_path.display())
            })?;
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

        let data_path = {
            let mut path = PathBuf::from(game_path);
            if game_is_mv {
                path.push("www");
            }
            path.push("data");
            path
        };

        Ok(Self {
            game_path: game_path.to_path_buf(),
            game_is_mv,
            font_name,
            font_size,
            screen_width,
            data_path,
            extra_single_text_codes: HashSet::new(),
            extra_text_codes: HashSet::new(),
        })
    }

    fn load_font(&self) -> anyhow::Result<Font> {
        let font_path = {
            let mut font_path = PathBuf::from(&self.game_path);
            if self.game_is_mv {
                font_path.push("www");
            }
            font_path.push("fonts");
            font_path.push(&self.font_name);
            font_path
        };
        let font = load_font(&font_path)?;

        Ok(font)
    }
}

/// Check lines for text overflow in a game.
pub fn check_line_size(options: &CheckLineSizeOptions) -> anyhow::Result<CheckLineSizeIter> {
    let font = options.load_font()?;
    let context = CheckLineSizeContext::new(
        font,
        options.font_size,
        options.screen_width,
        options.extra_single_text_codes.clone(),
        options.extra_text_codes.clone(),
    );
    let iter = CheckLineSizeIter::new(&options.data_path, context)?;

    Ok(iter)
}
