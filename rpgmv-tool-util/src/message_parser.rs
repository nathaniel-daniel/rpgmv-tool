use anyhow::bail;
use std::borrow::Cow;
use std::collections::HashSet;

#[derive(Debug, PartialEq)]
pub enum MessageNode<'a> {
    Text {
        value: Cow<'a, str>,
    },
    TextCode {
        name: Cow<'a, str>,
    },
    TextCodeWithBody {
        name: Cow<'a, str>,
        body: Cow<'a, str>,
    },
    YepTextCodeWithBody {
        name: Cow<'a, str>,
        body: Cow<'a, str>,
    },
}

#[derive(Debug)]
enum MessageParserState<'a> {
    Normal {
        start_index: Option<usize>,
    },
    TextCodeName {
        start_index: Option<usize>,
    },
    TextCodeBody {
        start_index: Option<usize>,
        name: &'a str,
        yep_message_core: bool,
    },
}

pub struct MessageParser<'a> {
    input: &'a str,
    char_iter: std::str::CharIndices<'a>,
    state: MessageParserState<'a>,
    yep_message_core: bool,
    single_text_codes: HashSet<String>,
    text_codes: HashSet<String>,
    yep_text_codes: HashSet<String>,
}

impl<'a> MessageParser<'a> {
    /// Make a new message parser to parse some text.
    pub fn new(input: &'a str) -> Self {
        let parser = Self {
            input,
            char_iter: input.char_indices(),
            state: MessageParserState::Normal { start_index: None },
            yep_message_core: false,
            single_text_codes: HashSet::new(),
            text_codes: HashSet::new(),
            yep_text_codes: HashSet::new(),
        };

        parser.add_rpgmaker_mv_text_codes()
    }

    /// Add RPGMaker MV text codes.
    ///
    /// See: https://www.yanfly.moe/wiki/Category:Text_Codes_(MV)
    pub fn add_rpgmaker_mv_text_codes(mut self) -> Self {
        // RPGMaker MV Defaults
        // Single
        self.single_text_codes.insert("g".to_string());
        self.single_text_codes.insert("!".to_string());
        self.single_text_codes.insert("{".to_string());
        self.single_text_codes.insert("}".to_string());
        self.single_text_codes.insert("|".to_string());
        self.single_text_codes.insert("<".to_string());
        self.single_text_codes.insert(">".to_string());
        // \C
        // This changes the color of future text to color 0,
        // based on window skin.
        //
        // This is actually the text code \C[n],
        // but a bug in RPGMaker makes it accept this as a single text code as well.
        self.single_text_codes.insert("c".to_string());
        // \.
        // Wait for 1/4 second.
        self.single_text_codes.insert(".".to_string());
        // \^
        // Do not wait for input after showing the text.
        self.single_text_codes.insert("^".to_string());

        // Body
        // \C[n]
        // This changes the color of future text to color n,
        // based on window skin.
        self.text_codes.insert("c".to_string());
        self.text_codes.insert("i".to_string());
        self.text_codes.insert("v".to_string());
        // \N[n]
        // This is replaced with the name of actor n.
        // TODO: Allow user to specify filler?
        self.text_codes.insert("n".to_string());

        // YEP_MessageCore
        // \n<x>
        // This creates a name box with contents x on the left side on top of the message box.
        self.yep_text_codes.insert("n".to_string());

        self
    }

    /// Enable support for parsing YEP_MessageCore.js messages.
    pub fn yep_message_core(mut self, value: bool) -> Self {
        self.yep_message_core = value;
        self
    }

    /// Add YEP_MessageCore.js text codes.
    ///
    /// See: https://www.yanfly.moe/wiki/Category:Text_Codes_(MV)
    pub fn add_yep_message_core_text_codes(mut self) -> Self {
        // Single
        // \fr
        // Resets all font changes.
        self.single_text_codes.insert("fr".to_string());
        // \fb
        // Toggles font boldness.
        self.single_text_codes.insert("fb".to_string());
        // \fi
        // Toggles font italic.
        self.single_text_codes.insert("fi".to_string());

        // Body
        // \is[n]
        // Writes out skill n's name including icon.
        self.text_codes.insert("is".to_string());

        self
    }

    /// Add a new single text code to the parser.
    pub fn add_single_text_code(&mut self, single_text_code: &str) {
        self.single_text_codes
            .insert(single_text_code.to_ascii_lowercase());
    }

    /// Add a new text code to the parser.
    pub fn add_text_code(&mut self, text_code: &str) {
        self.text_codes.insert(text_code.to_ascii_lowercase());
    }

    fn create_text_node(&self, start: usize, end: usize) -> MessageNode<'a> {
        MessageNode::Text {
            value: Cow::Borrowed(&self.input[start..end]),
        }
    }

    fn process_normal(
        &mut self,
        ch_index: usize,
        ch: char,
        start_index: usize,
        nodes: &mut Vec<MessageNode<'a>>,
    ) -> bool {
        if ch == '\\' {
            if start_index != ch_index {
                nodes.push(self.create_text_node(start_index, ch_index));
            }
            self.state = MessageParserState::TextCodeName { start_index: None };
        }

        true
    }

    fn process_text_code_name(
        &mut self,
        ch_index: usize,
        ch: char,
        start_index: usize,
        nodes: &mut Vec<MessageNode<'a>>,
    ) -> anyhow::Result<bool> {
        let text_code = &self.input[start_index..ch_index];
        if ch == '[' {
            let text_code_lower = text_code.to_ascii_lowercase();
            if !self.text_codes.contains(&text_code_lower) {
                bail!("Unknown text code \"{text_code}\"");
            }

            self.state = MessageParserState::TextCodeBody {
                name: text_code,
                start_index: None,
                yep_message_core: false,
            };

            Ok(true)
        } else if self.yep_message_core && ch == '<' {
            let text_code_lower = text_code.to_ascii_lowercase();
            if !self.yep_text_codes.contains(&text_code_lower) {
                bail!("Unknown yep text code \"{text_code}\"");
            }

            self.state = MessageParserState::TextCodeBody {
                start_index: None,
                name: text_code,
                yep_message_core: true,
            };

            Ok(true)
        } else if self
            .single_text_codes
            .contains(&text_code.to_ascii_lowercase())
        {
            nodes.push(MessageNode::TextCode {
                name: Cow::Borrowed(text_code),
            });
            self.state = MessageParserState::Normal {
                // The current char we are on is part of the next state.
                start_index: Some(ch_index),
            };

            Ok(false)
        } else {
            Ok(true)
        }
    }

    fn process_text_code_body(
        &mut self,
        ch_index: usize,
        ch: char,
        start_index: usize,
        name: &'a str,
        yep_message_core: bool,
        nodes: &mut Vec<MessageNode<'a>>,
    ) -> anyhow::Result<bool> {
        if ch == ']' {
            let body = &self.input[start_index..ch_index];

            nodes.push(MessageNode::TextCodeWithBody {
                name: (*name).into(),
                body: body.into(),
            });
            self.state = MessageParserState::Normal { start_index: None };
        } else if yep_message_core && ch == '>' {
            let body = &self.input[start_index..ch_index];

            nodes.push(MessageNode::YepTextCodeWithBody {
                name: (*name).into(),
                body: body.into(),
            });
            self.state = MessageParserState::Normal { start_index: None };
        }

        Ok(true)
    }

    pub fn parse(&mut self) -> anyhow::Result<Vec<MessageNode<'a>>> {
        let mut nodes = Vec::new();
        let mut next_char_entry = self.char_iter.next();
        while let Some((ch_index, ch)) = next_char_entry {
            let consume = match &mut self.state {
                MessageParserState::Normal { start_index } => {
                    let start_index = *start_index.get_or_insert(ch_index);
                    self.process_normal(ch_index, ch, start_index, &mut nodes)
                }
                MessageParserState::TextCodeName { start_index } => {
                    let start_index = *start_index.get_or_insert(ch_index);

                    self.process_text_code_name(ch_index, ch, start_index, &mut nodes)?
                }
                MessageParserState::TextCodeBody {
                    start_index,
                    name,
                    yep_message_core,
                } => {
                    let start_index = *start_index.get_or_insert(ch_index);
                    let name = *name;
                    let yep_message_core = *yep_message_core;

                    self.process_text_code_body(
                        ch_index,
                        ch,
                        start_index,
                        name,
                        yep_message_core,
                        &mut nodes,
                    )?
                }
            };

            if consume {
                next_char_entry = self.char_iter.next();
            }
        }

        match self.state {
            MessageParserState::Normal { start_index } => {
                if let Some(start_index) = start_index {
                    nodes.push(self.create_text_node(start_index, self.input.len()));
                }
            }
            MessageParserState::TextCodeName { start_index } => {
                let start_index = match start_index {
                    Some(start_index) => start_index,
                    // We parsed a \, but then the text ended.
                    None => bail!("Incomplete text code"),
                };
                let text_code = &self.input[start_index..];

                // TODO: Is this possible?
                if text_code.is_empty() {
                    bail!("Incomplete text code");
                }

                if self
                    .single_text_codes
                    .contains(&text_code.to_ascii_lowercase())
                {
                    nodes.push(MessageNode::TextCode {
                        name: Cow::Borrowed(text_code),
                    });
                } else {
                    bail!("Unknown single text code \"{text_code}\"");
                }
            }
            _ => bail!("Invalid state at end of string, got \"{:?}\"", self.state),
        };

        Ok(nodes)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn yep() {
        let tests = [
            (
                "\\n<Bob>Hello!",
                vec![
                    MessageNode::YepTextCodeWithBody {
                        name: "n".into(),
                        body: "Bob".into(),
                    },
                    MessageNode::Text {
                        value: "Hello!".into(),
                    },
                ],
            ),
            (
                "\\fbThis is bold.\\fiThis is italic.\\frThis is normal.",
                vec![
                    MessageNode::TextCode { name: "fb".into() },
                    MessageNode::Text {
                        value: "This is bold.".into(),
                    },
                    MessageNode::TextCode { name: "fi".into() },
                    MessageNode::Text {
                        value: "This is italic.".into(),
                    },
                    MessageNode::TextCode { name: "fr".into() },
                    MessageNode::Text {
                        value: "This is normal.".into(),
                    },
                ],
            ),
        ];

        for (input, expected_output) in tests {
            let mut parser = MessageParser::new(input)
                .yep_message_core(true)
                .add_yep_message_core_text_codes();
            let actual_output = parser.parse().expect("Failed to parse");
            // dbg!(&actual_output);
            assert!(
                actual_output == expected_output,
                "actual != expected, {actual_output:#?} != {expected_output:#?}"
            );
        }
    }

    #[test]
    fn strip_escapes() {
        let tests = [
            (
                "\\C[2]Colored",
                vec![
                    MessageNode::TextCodeWithBody {
                        name: "C".into(),
                        body: "2".into(),
                    },
                    MessageNode::Text {
                        value: "Colored".into(),
                    },
                ],
            ),
            ("", vec![]),
            (" ", vec![MessageNode::Text { value: " ".into() }]),
            (
                "Colored\\C[2]",
                vec![
                    MessageNode::Text {
                        value: "Colored".into(),
                    },
                    MessageNode::TextCodeWithBody {
                        name: "C".into(),
                        body: "2".into(),
                    },
                ],
            ),
            (
                "\\G abc",
                vec![
                    MessageNode::TextCode { name: "G".into() },
                    MessageNode::Text {
                        value: " abc".into(),
                    },
                ],
            ),
            ("\\G", vec![MessageNode::TextCode { name: "G".into() }]),
            (
                "\\V[1] potions",
                vec![
                    MessageNode::TextCodeWithBody {
                        name: "V".into(),
                        body: "1".into(),
                    },
                    MessageNode::Text {
                        value: " potions".into(),
                    },
                ],
            ),
            (
                "\\C[2]Colored\\C",
                vec![
                    MessageNode::TextCodeWithBody {
                        name: "C".into(),
                        body: "2".into(),
                    },
                    MessageNode::Text {
                        value: "Colored".into(),
                    },
                    MessageNode::TextCode { name: "C".into() },
                ],
            ),
            (
                "\\C[2]Colored\\C ",
                vec![
                    MessageNode::TextCodeWithBody {
                        name: "C".into(),
                        body: "2".into(),
                    },
                    MessageNode::Text {
                        value: "Colored".into(),
                    },
                    MessageNode::TextCode { name: "C".into() },
                    MessageNode::Text { value: " ".into() },
                ],
            ),
            (
                "\\^nowait",
                vec![
                    MessageNode::TextCode { name: "^".into() },
                    MessageNode::Text {
                        value: "nowait".into(),
                    },
                ],
            ),
            (
                "\\.\\.test",
                vec![
                    MessageNode::TextCode { name: ".".into() },
                    MessageNode::TextCode { name: ".".into() },
                    MessageNode::Text {
                        value: "test".into(),
                    },
                ],
            ),
        ];

        for (input, expected_output) in tests {
            let mut parser = MessageParser::new(input);
            let actual_output = parser.parse().expect("Failed to parse");
            // dbg!(&actual_output);
            assert!(actual_output == expected_output);
        }
    }
}
