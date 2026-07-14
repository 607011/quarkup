use crate::ast::{BlockNode, Document, InlineNode, LatticeCell, LatticeRow, ListItem, RowType};
use crate::lexer::{Flavor, Lexer, Token};

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    lookahead: Option<Token<'a>>,
}

impl<'a> Parser<'a> {
    pub fn new(mut lexer: Lexer<'a>) -> Self {
        let lookahead = lexer.next();
        Self { lexer, lookahead }
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
        match self.peek()? {
            Token::Quark { flavor, count } => {
                let flavor = *flavor;
                let count = *count;
                self.consume();

                match flavor {
                    Flavor::Up => {
                        let content = self.parse_inline_until_line_end();
                        Some(BlockNode::Heading {
                            level: count,
                            content,
                        })
                    }
                    Flavor::Top => {
                        let raw_line = self.collect_raw_line_content();
                        let mut parts = raw_line.split_whitespace();
                        let key = parts.next()?.to_string();
                        let value = parts.collect::<Vec<&str>>().join(" ");
                        Some(BlockNode::Metadata { key, value })
                    }
                    Flavor::Graphic => {
                        let raw_line = self.collect_raw_line_content();
                        let mut parts = raw_line.split_whitespace();
                        let path = parts.next()?.to_string();
                        let caption = if parts.clone().count() > 0 {
                            Some(parts.collect::<Vec<&str>>().join(" "))
                        } else {
                            None
                        };
                        Some(BlockNode::Image { path, caption })
                    }
                    Flavor::Lattice if count == 1 => {
                        self.collect_raw_line_content(); // Consume residual line-break or spaces after .l
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
                        Some(BlockNode::Lattice(rows))
                    }
                    Flavor::Strange => {
                        let raw_line = self.collect_raw_line_content();
                        let modifier = raw_line.trim().to_string();
                        let is_math = modifier == "math";

                        let mut lines = Vec::new();

                        while self.peek().is_some() {
                            // We inspect the raw line first to check for escapings or block termination
                            let next_line = self.collect_raw_line_content();
                            let trimmed = next_line.trim();

                            if trimmed == ".." || trimmed.starts_with("..") {
                                // Plain annihilator found: terminate the block
                                break;
                            } else if trimmed == "\\.." || trimmed.starts_with("\\..") {
                                // Escaped annihilator: remove the backslash and keep the dots
                                let unescaped = next_line.replacen("\\..", "..", 1);
                                lines.push(unescaped);
                            } else {
                                lines.push(next_line);
                            }
                        }

                        if is_math {
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
                        }
                    }
                    _ => {
                        let content = self.parse_inline_until_line_end();
                        Some(BlockNode::Paragraph(content))
                    }
                }
            }
            _ => {
                let content = self.parse_inline_until_line_end();
                if content.is_empty() {
                    None
                } else {
                    Some(BlockNode::Paragraph(content))
                }
            }
        }
    }

    fn parse_lattice_row(&self, raw_line: &str) -> Option<LatticeRow> {
        let trimmed = raw_line.trim();
        if trimmed.is_empty() {
            return None;
        }

        let mut row_type = RowType::Body;
        let mut content_part = trimmed;

        if trimmed.starts_with("h:") {
            row_type = RowType::Header;
            content_part = &trimmed[2..];
        } else if trimmed.starts_with("f:") {
            row_type = RowType::Footer;
            content_part = &trimmed[2..];
        } else if trimmed.starts_with("s:") {
            row_type = RowType::Section;
            content_part = &trimmed[2..];
        }

        let raw_cells: Vec<&str> = content_part.split(';').collect();
        let mut cells = Vec::new();

        for raw_cell in raw_cells {
            let cell_trimmed = raw_cell.trim();
            let is_colspan_marker = cell_trimmed == ">";
            let is_rowspan_marker = cell_trimmed == "_";

            let cell_lexer = Lexer::new(raw_cell);
            let mut cell_parser = Parser::new(cell_lexer);
            let content = cell_parser.parse_inline_until_line_end();

            cells.push(LatticeCell {
                content,
                colspan: 1,
                rowspan: 1,
                is_merged: false,
                is_colspan_marker,
                is_rowspan_marker,
            });
        }

        Some(LatticeRow { row_type, cells })
    }

    fn parse_list_block(&mut self) -> Option<BlockNode> {
        let first_tok = self.peek()?;
        let ordered = match first_tok {
            Token::Quark {
                flavor: Flavor::Electron,
                ..
            } => true,
            _ => false,
        };

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
                        if raw_text.starts_with("math ") {
                            let formula = raw_text["math ".len()..].to_string();
                            nodes.push(InlineNode::InlineMath(formula));
                        } else {
                            nodes.push(InlineNode::Formatted {
                                flavor,
                                content: inner,
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
                        if raw_text.starts_with("math ") {
                            let formula = raw_text["math ".len()..].to_string();
                            nodes.push(InlineNode::InlineMath(formula));
                        } else {
                            nodes.push(InlineNode::Formatted {
                                flavor,
                                content: inner,
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
            }
        }
        s
    }
}
