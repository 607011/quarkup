use crate::ast::{
    BlockNode, CellAlign, Document, InlineNode, LatticeCell, LatticeRow, ListItem, RowType,
};
use crate::lexer::{Flavor, Lexer, Token};
use std::collections::HashMap;

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    lookahead: Option<Token<'a>>,
    defines: HashMap<String, String>,
}

impl<'a> Parser<'a> {
    pub fn new(mut lexer: Lexer<'a>, defines: HashMap<String, String>) -> Self {
        let lookahead = lexer.next();
        Self {
            lexer,
            lookahead,
            defines,
        }
    }

    fn consume(&mut self) -> Option<Token<'a>> {
        let current = self.lookahead;
        self.lookahead = self.lexer.next();
        current
    }

    fn peek(&self) -> Option<&Token<'a>> {
        self.lookahead.as_ref()
    }

    pub fn parse(mut self) -> Document {
        let mut blocks = Vec::new();

        while self.peek().is_some() {
            if let Some(Token::LineBreak) | Some(Token::Space) = self.peek() {
                self.consume();
                continue;
            }

            if let Some(Token::Quark {
                flavor: Flavor::Neutrino,
                ..
            })
            | Some(Token::Quark {
                flavor: Flavor::Electron,
                ..
            }) = self.peek()
            {
                if let Some(list_block) = self.parse_list_block() {
                    blocks.push(list_block);
                }
                continue;
            }

            if let Some(block) = self.parse_block() {
                blocks.push(block);
            }
        }

        Document { blocks }
    }

    fn parse_block(&mut self) -> Option<BlockNode> {
        if let Some(Token::Quark { flavor, count }) = self.peek() {
            let flavor = *flavor;
            let count = *count;

            match flavor {
                Flavor::Up => {
                    self.consume();
                    let content = self.parse_inline_until_line_end();
                    return Some(BlockNode::Heading {
                        level: count,
                        content,
                    });
                }
                Flavor::Top => {
                    self.consume();
                    let raw_line = self.collect_raw_line_content();
                    let mut parts = raw_line.split_whitespace();
                    return if let Some(key) = parts.next() {
                        let value = parts.collect::<Vec<&str>>().join(" ");
                        // Register metadata dynamically inside the parser environment
                        self.defines.insert(key.to_string(), value.clone());
                        Some(BlockNode::Metadata {
                            key: key.to_string(),
                            value,
                        })
                    } else {
                        None
                    };
                }
                Flavor::Graphic => {
                    self.consume();
                    let raw_line = self.collect_raw_line_content();
                    let mut parts = raw_line.split_whitespace();
                    let path = parts.next()?.to_string();
                    let caption = if parts.clone().count() > 0 {
                        Some(parts.collect::<Vec<&str>>().join(" "))
                    } else {
                        None
                    };
                    return Some(BlockNode::Image { path, caption });
                }
                Flavor::Lattice if count == 1 => {
                    self.consume();
                    self.collect_raw_line_content();
                    let mut rows = Vec::new();

                    while let Some(tok) = self.peek() {
                        if let Token::Annihilator = tok {
                            self.consume();
                            break;
                        }
                        let raw_row = self.collect_raw_line_content();
                        if let Some(lattice_row) = self.parse_lattice_row(&raw_row) {
                            rows.push(lattice_row);
                        }
                    }
                    return Some(BlockNode::Lattice(rows));
                }
                Flavor::Strange => {
                    self.consume();
                    let raw_line = self.collect_raw_line_content();
                    let modifier = raw_line.trim().to_string();
                    let is_math = modifier == "math";

                    let mut lines = Vec::new();

                    while self.peek().is_some() {
                        let next_line = self.collect_raw_line_content();
                        let trimmed = next_line.trim();

                        if trimmed == ".." || trimmed.starts_with("..") {
                            break;
                        } else if trimmed == "\\.." || trimmed.starts_with("\\..") {
                            let unescaped = next_line.replacen("\\..", "..", 1);
                            lines.push(unescaped);
                        } else {
                            lines.push(next_line);
                        }
                    }

                    return if is_math {
                        Some(BlockNode::MathBlock(lines.join("\n")))
                    } else {
                        let language = if modifier.is_empty() {
                            None
                        } else {
                            Some(modifier)
                        };
                        Some(BlockNode::CodeBlock {
                            language,
                            code: lines.join("\n"),
                        })
                    };
                }
                Flavor::Bottom if count == 1 => {
                    self.consume();
                    let raw_line = self.collect_raw_line_content();

                    return if self.evaluate_condition(&raw_line) {
                        let mut inner_blocks = Vec::new();
                        while let Some(tok) = self.peek() {
                            if let Token::Annihilator = tok {
                                self.consume();
                                break;
                            }
                            if let Some(block) = self.parse_block() {
                                inner_blocks.push(block);
                            }
                        }
                        Some(BlockNode::Conditional(inner_blocks))
                    } else {
                        // Safe skipping algorithm protecting nested block structures
                        let mut depth = 1;
                        while depth > 0 {
                            if let Some(tok) = self.peek() {
                                match tok {
                                    Token::Annihilator => {
                                        self.consume();
                                        depth -= 1;
                                    }
                                    Token::Quark { flavor, count }
                                        if *count == 1
                                            && (*flavor == Flavor::Lattice
                                                || *flavor == Flavor::Strange
                                                || *flavor == Flavor::Bottom) =>
                                    {
                                        self.collect_raw_line_content();
                                        depth += 1;
                                    }
                                    _ => {
                                        self.collect_raw_line_content();
                                    }
                                }
                            } else {
                                break;
                            }
                        }
                        None
                    };
                }
                // Any other flavor (Charm, Muon, Down, ...) is itself inline
                // content, e.g. a paragraph that opens with `.m bold text..`.
                // Don't consume it here — parse_inline_until_line_end needs
                // to see it to wrap it in the right InlineNode::Formatted.
                _ => {}
            }
        }

        let content = self.parse_inline_until_line_end();
        if content.is_empty() {
            None
        } else {
            Some(BlockNode::Paragraph(content))
        }
    }

    fn parse_lattice_row(&self, raw_line: &str) -> Option<LatticeRow> {
        let trimmed = raw_line.trim();
        if trimmed.is_empty() {
            return None;
        }

        let (row_type, content_part) = if let Some(rest) = trimmed.strip_prefix("h:") {
            (RowType::Header, rest)
        } else if let Some(rest) = trimmed.strip_prefix("f:") {
            (RowType::Footer, rest)
        } else if let Some(rest) = trimmed.strip_prefix("s:") {
            (RowType::Section, rest)
        } else {
            (RowType::Body, trimmed)
        };

        let raw_cells: Vec<&str> = content_part.split(';').collect();
        let mut cells = Vec::new();

        for raw_cell in raw_cells {
            let cell_trimmed = raw_cell.trim();
            let is_colspan_marker = cell_trimmed == ">";
            let is_rowspan_marker = cell_trimmed == "_";

            // A leading ".^ " / ".> " marks cell alignment; stripped before
            // the cell's own content is lexed so it never shows up as text.
            let (align, cell_source) = if let Some(rest) = cell_trimmed.strip_prefix(".^ ") {
                (CellAlign::Center, rest)
            } else if let Some(rest) = cell_trimmed.strip_prefix(".> ") {
                (CellAlign::Right, rest)
            } else {
                (CellAlign::Left, raw_cell)
            };

            let cell_lexer = Lexer::new(cell_source);
            let mut cell_parser = Parser::new(cell_lexer, HashMap::new());
            let content = cell_parser.parse_inline_until_line_end();

            cells.push(LatticeCell {
                content,
                colspan: 1,
                rowspan: 1,
                is_merged: false,
                is_colspan_marker,
                is_rowspan_marker,
                align,
            });
        }

        Some(LatticeRow { row_type, cells })
    }

    fn parse_list_block(&mut self) -> Option<BlockNode> {
        let first_tok = self.peek()?;
        let ordered = matches!(
            first_tok,
            Token::Quark {
                flavor: Flavor::Electron,
                ..
            }
        );

        let mut items = Vec::new();

        while let Some(Token::Quark { flavor, count }) = self.peek() {
            let current_ordered = match flavor {
                Flavor::Electron => true,
                Flavor::Neutrino => false,
                _ => break,
            };

            if current_ordered != ordered {
                break;
            }

            let level = *count;
            self.consume();

            let content = self.parse_inline_until_line_end();
            items.push(ListItem { level, content });

            while let Some(Token::LineBreak) = self.peek() {
                self.consume();
            }
        }

        Some(BlockNode::List { ordered, items })
    }

    fn collect_raw_line_content(&mut self) -> String {
        let mut buffer = String::new();
        while let Some(tok) = self.peek() {
            match tok {
                Token::LineBreak => {
                    self.consume();
                    break;
                }
                Token::Text(t) => {
                    buffer.push_str(t);
                    self.consume();
                }
                Token::Space => {
                    buffer.push(' ');
                    self.consume();
                }
                Token::Annihilator => {
                    buffer.push_str("..");
                    self.consume();
                }
                Token::Quark { flavor, count } => {
                    let letter = match flavor {
                        Flavor::Up => 'u',
                        Flavor::Down => 'd',
                        Flavor::Charm => 'c',
                        Flavor::Strange => 's',
                        Flavor::Top => 't',
                        Flavor::Bottom => 'b',
                        Flavor::Graphic => 'g',
                        Flavor::Neutrino => 'n',
                        Flavor::Electron => 'e',
                        Flavor::Lattice => 'l',
                        Flavor::Muon => 'm',
                    };
                    buffer.push('.');
                    for _ in 0..*count {
                        buffer.push(letter);
                    }
                    buffer.push(' ');
                    self.consume();
                }
            }
        }
        buffer
    }

    fn parse_inline_until_line_end(&mut self) -> Vec<InlineNode> {
        let mut nodes = Vec::new();

        while let Some(tok) = self.peek() {
            match tok {
                Token::LineBreak => {
                    self.consume();
                    break;
                }
                Token::Annihilator => {
                    nodes.push(InlineNode::Text("..".to_string()));
                    self.consume();
                }
                Token::Space => {
                    nodes.push(InlineNode::Text(" ".to_string()));
                    self.consume();
                }
                Token::Text(t) => {
                    nodes.push(InlineNode::Text(t.to_string()));
                    self.consume();
                }
                Token::Quark { flavor, count: _ } => {
                    let flavor = *flavor;
                    self.consume();

                    if flavor == Flavor::Strange {
                        let inner = self.parse_inline_until_annihilator();
                        let raw_text = self.nodes_to_string(&inner);
                        if let Some(formula) = raw_text.strip_prefix("math ") {
                            nodes.push(InlineNode::InlineMath(formula.to_string()));
                        } else {
                            nodes.push(InlineNode::Formatted {
                                flavor,
                                content: inner,
                            });
                        }
                    } else if flavor == Flavor::Bottom {
                        // 1. Wir parsen den gesamten Inhalt bis zum passenden schließenden Annihilator `..`
                        let inner = self.parse_inline_until_annihilator();
                        let raw_text = self.nodes_to_string(&inner);

                        if let Some(pos) = raw_text.find('|') {
                            let (cond, _) = raw_text.split_at(pos);

                            if self.evaluate_condition(cond) {
                                // If the condition is TRUE, we filter out the condition ("target=web|")
                                // from the already parsed inner nodes and add them to the AST.
                                let mut condition_prefix_len = cond.len() + 1; // Condition + '|'

                                for node in inner {
                                    match node {
                                        InlineNode::Text(mut t) => {
                                            if condition_prefix_len > 0 {
                                                if t.len() >= condition_prefix_len {
                                                    t.drain(0..condition_prefix_len);
                                                    condition_prefix_len = 0;
                                                    if !t.is_empty() {
                                                        nodes.push(InlineNode::Text(t));
                                                    }
                                                } else {
                                                    condition_prefix_len -= t.len();
                                                }
                                            } else {
                                                nodes.push(InlineNode::Text(t));
                                            }
                                        }
                                        _ => {
                                            // Complex nodes (like our InlineNode::Link!) are preserved as-is!
                                            if condition_prefix_len == 0 {
                                                nodes.push(node);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    } else if flavor == Flavor::Lattice {
                        let inner = self.parse_inline_until_annihilator();
                        let raw_text = self.nodes_to_string(&inner);
                        if let Some(pos) = raw_text.find('|') {
                            let (url, text_part) = raw_text.split_at(pos);
                            let text_content = &text_part[1..]; // skip '|'

                            let mut sub_parser =
                                Parser::new(Lexer::new(text_content), self.defines.clone());
                            let parsed_text = sub_parser.parse_inline_until_line_end();

                            nodes.push(InlineNode::Link {
                                url: url.trim().to_string(),
                                text: parsed_text,
                            });
                        } else {
                            nodes.push(InlineNode::Link {
                                url: raw_text.trim().to_string(),
                                text: vec![InlineNode::Text(raw_text)],
                            });
                        }
                    } else {
                        let content = self.parse_inline_until_annihilator();
                        nodes.push(InlineNode::Formatted { flavor, content });
                    }
                }
            }
        }

        nodes
    }

    fn parse_inline_until_annihilator(&mut self) -> Vec<InlineNode> {
        let mut nodes = Vec::new();

        while let Some(tok) = self.peek() {
            match tok {
                Token::Annihilator => {
                    self.consume();
                    break;
                }
                Token::LineBreak => {
                    self.consume();
                    break;
                }
                Token::Space => {
                    nodes.push(InlineNode::Text(" ".to_string()));
                    self.consume();
                }
                Token::Text(t) => {
                    nodes.push(InlineNode::Text(t.to_string()));
                    self.consume();
                }
                Token::Quark { flavor, count: _ } => {
                    let flavor = *flavor;
                    self.consume();

                    if flavor == Flavor::Strange {
                        let inner = self.parse_inline_until_annihilator();
                        let raw_text = self.nodes_to_string(&inner);
                        if let Some(formula) = raw_text.strip_prefix("math ") {
                            nodes.push(InlineNode::InlineMath(formula.to_string()));
                        } else {
                            nodes.push(InlineNode::Formatted {
                                flavor,
                                content: inner,
                            });
                        }
                    } else if flavor == Flavor::Bottom {
                        let inner = self.parse_inline_until_annihilator();
                        let raw_text = self.nodes_to_string(&inner);

                        if let Some(pos) = raw_text.find('|') {
                            let (cond, _) = raw_text.split_at(pos);
                            if self.evaluate_condition(cond) {
                                let mut condition_prefix_len = cond.len() + 1; // condition + '|'

                                for node in inner {
                                    match node {
                                        InlineNode::Text(mut t) => {
                                            if condition_prefix_len > 0 {
                                                if t.len() >= condition_prefix_len {
                                                    t.drain(0..condition_prefix_len);
                                                    condition_prefix_len = 0;
                                                    if !t.is_empty() {
                                                        nodes.push(InlineNode::Text(t));
                                                    }
                                                } else {
                                                    condition_prefix_len -= t.len();
                                                }
                                            } else {
                                                nodes.push(InlineNode::Text(t));
                                            }
                                        }
                                        _ => {
                                            if condition_prefix_len == 0 {
                                                nodes.push(node);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    } else if flavor == Flavor::Lattice {
                        let inner = self.parse_inline_until_annihilator();
                        let raw_text = self.nodes_to_string(&inner);
                        if let Some(pos) = raw_text.find('|') {
                            let (url, text_part) = raw_text.split_at(pos);
                            let text_content = &text_part[1..]; // skip '|'

                            let mut sub_parser =
                                Parser::new(Lexer::new(text_content), self.defines.clone());
                            let parsed_text = sub_parser.parse_inline_until_line_end();

                            nodes.push(InlineNode::Link {
                                url: url.trim().to_string(),
                                text: parsed_text,
                            });
                        } else {
                            nodes.push(InlineNode::Link {
                                url: raw_text.trim().to_string(),
                                text: vec![InlineNode::Text(raw_text)],
                            });
                        }
                    } else {
                        let content = self.parse_inline_until_annihilator();
                        nodes.push(InlineNode::Formatted { flavor, content });
                    }
                }
            }
        }

        nodes
    }

    fn nodes_to_string(&self, nodes: &[InlineNode]) -> String {
        let mut s = String::new();
        for node in nodes {
            match node {
                InlineNode::Text(t) => s.push_str(t),
                InlineNode::Formatted { content, .. } => s.push_str(&self.nodes_to_string(content)),
                InlineNode::InlineMath(m) => {
                    s.push_str("math ");
                    s.push_str(m);
                }
                InlineNode::Link { text, .. } => {
                    s.push_str(&self.nodes_to_string(text));
                }
            }
        }
        s
    }

    // Helper to evaluate a condition string against active definitions (supports '=' and whitespace)
    fn evaluate_condition(&self, cond: &str) -> bool {
        let trimmed_cond = cond.trim();
        let is_negated = trimmed_cond.starts_with('!');
        let core = if is_negated {
            trimmed_cond[1..].trim()
        } else {
            trimmed_cond
        };

        // Parse key and expected value, supporting both 'key=value' and 'key value'
        let (key, expected_value) = if let Some(pos) = core.find('=') {
            let (k, v) = core.split_at(pos);
            (k.trim(), Some(v[1..].trim()))
        } else {
            let mut parts = core.split_whitespace();
            let k = parts.next().unwrap_or("");
            let v = parts.next();
            (k, v)
        };

        let actual_value = self.defines.get(key);
        let mut matches = match expected_value {
            Some(val) => actual_value.map(|v| v == val).unwrap_or(false),
            None => actual_value
                .map(|v| v != "false" && !v.is_empty())
                .unwrap_or(false),
        };

        if is_negated {
            matches = !matches;
        }
        matches
    }
}
