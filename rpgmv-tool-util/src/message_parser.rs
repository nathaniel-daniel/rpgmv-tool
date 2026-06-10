use anyhow::bail;
use std::borrow::Cow;
use std::collections::HashSet;

#[derive(Debug, PartialEq)]
pub enum MessageNode<'a> {
    Text {
        value: Cow<'a, str>,
    },
    TextCode {
        name: char,
    },
    TextCodeWithBody {
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
        name: &'a str,
        start_index: Option<usize>,
    },
}

pub struct MessageParser<'a> {
    input: &'a str,
    char_iter: std::str::CharIndices<'a>,
    state: MessageParserState<'a>,
    single_text_codes: HashSet<char>,
    text_codes: HashSet<String>,
}

impl<'a> MessageParser<'a> {
    pub fn new(input: &'a str) -> Self {
        let mut parser = Self {
            input,
            char_iter: input.char_indices(),
            state: MessageParserState::Normal { start_index: None },
            single_text_codes: HashSet::new(),
            text_codes: HashSet::new(),
        };

        // RPGMaker MV Defaults
        // Single
        parser.single_text_codes.insert('g');
        parser.single_text_codes.insert('!');
        parser.single_text_codes.insert('{');
        parser.single_text_codes.insert('}');
        parser.single_text_codes.insert('|');
        parser.single_text_codes.insert('<');
        parser.single_text_codes.insert('>');
        // \C
        // This changes the color of future text to color 0,
        // based on window skin.
        //
        // This is actually the text code \C[n],
        // but a bug in RPGMaker makes it accept this as a single text code as well.
        parser.single_text_codes.insert('c');
        // \.
        // Wait for 1/4 second.
        parser.single_text_codes.insert('.');

        // Body
        // \C[n]
        // This changes the color of future text to color n,
        // based on window skin.
        parser.text_codes.insert("c".to_string());
        parser.text_codes.insert("i".to_string());
        parser.text_codes.insert("v".to_string());
        // \N[n]
        // This is replaced with the name of actor n.
        // TODO: Allow user to specify filler?
        parser.text_codes.insert("n".to_string());

        parser
    }

    /// Add a new single text code to the parser.
    pub fn add_single_text_code(&mut self, single_text_code: char) {
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

    pub fn parse(&mut self) -> anyhow::Result<Vec<MessageNode<'a>>> {
        let mut nodes = Vec::new();
        while let Some((ch_index, ch)) = self.char_iter.next() {
            match self.state {
                MessageParserState::Normal { start_index } => {
                    let start_index = match start_index {
                        Some(start_index) => start_index,
                        None => {
                            self.state = MessageParserState::Normal {
                                start_index: Some(ch_index),
                            };
                            ch_index
                        }
                    };

                    if ch == '\\' {
                        if start_index != ch_index {
                            nodes.push(self.create_text_node(start_index, ch_index));
                        }
                        self.state = MessageParserState::TextCodeName { start_index: None };
                    }
                }
                MessageParserState::TextCodeName { start_index } => {
                    let start_index = match start_index {
                        Some(start_index) => start_index,
                        None => {
                            self.state = MessageParserState::TextCodeName {
                                start_index: Some(ch_index),
                            };
                            ch_index
                        }
                    };

                    let text_code = &self.input[start_index..ch_index];
                    if ch == '[' {
                        let text_code_lower = text_code.to_ascii_lowercase();
                        if !self.text_codes.contains(&text_code_lower) {
                            bail!("unknown text code \"{text_code}\"");
                        }

                        self.state = MessageParserState::TextCodeBody {
                            name: text_code,
                            start_index: None,
                        }
                    } else if text_code.len() == 1 {
                        let text_code_ch = text_code.chars().next().unwrap();

                        if self
                            .single_text_codes
                            .contains(&text_code_ch.to_ascii_lowercase())
                        {
                            nodes.push(MessageNode::TextCode { name: text_code_ch });
                            self.state = MessageParserState::Normal {
                                // The current char we are on is part of the next state.
                                start_index: Some(ch_index),
                            };
                        }
                    };
                }
                MessageParserState::TextCodeBody { name, start_index } => {
                    let start_index = match start_index {
                        Some(start_index) => start_index,
                        None => {
                            self.state = MessageParserState::TextCodeBody {
                                name,
                                start_index: Some(ch_index),
                            };
                            continue;
                        }
                    };

                    if ch == ']' {
                        let body = &self.input[start_index..ch_index];

                        nodes.push(MessageNode::TextCodeWithBody {
                            name: name.into(),
                            body: body.into(),
                        });
                        self.state = MessageParserState::Normal { start_index: None };
                    }
                }
            };
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
                    None => bail!("incomplete text code"),
                };
                let text_code = &self.input[start_index..];

                // TODO: Is this possible?
                if text_code.is_empty() {
                    bail!("incomplete text code");
                }

                let text_code_ch = text_code.chars().next().unwrap();

                // We parsed a \ and read chars until the string end, never finding a [.
                // This is likely an unknown single text code.
                if text_code.len() > 1 {
                    bail!("unknown single text code \"{text_code_ch}\"");
                }

                if self
                    .single_text_codes
                    .contains(&text_code_ch.to_ascii_lowercase())
                {
                    nodes.push(MessageNode::TextCode { name: text_code_ch });
                } else {
                    bail!("unknown single text code \"{text_code_ch}\"");
                }
            }
            _ => bail!("invalid state at end of string, got \"{:?}\"", self.state),
        };

        Ok(nodes)
    }
}

#[cfg(test)]
mod test {
    use super::*;

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
                    MessageNode::TextCode { name: 'G' },
                    MessageNode::Text {
                        value: " abc".into(),
                    },
                ],
            ),
            ("\\G", vec![MessageNode::TextCode { name: 'G' }]),
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
                    MessageNode::TextCode { name: 'C' },
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
                    MessageNode::TextCode { name: 'C' },
                    MessageNode::Text { value: " ".into() },
                ],
            ),
        ];

        for (input, expected_output) in tests {
            let mut parser = MessageParser::new(input);
            let actual_output = parser.parse().expect("failed to parse");
            dbg!(&actual_output);
            assert!(actual_output == expected_output);
        }
    }
}
