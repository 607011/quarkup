use std::iter::Peekable;
use std::str::CharIndices;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Flavor {
    Up,       // u
    Down,     // d
    Charm,    // c
    Strange,  // s
    Top,      // t
    Bottom,   // b
    Graphic,  // g
    Neutrino, // n
    Electron, // e
    Lattice,  // l
    Muon,     // m
}

impl Flavor {
    pub fn from_char(c: char) -> Option<Self> {
        match c {
            'u' => Some(Flavor::Up),
            'd' => Some(Flavor::Down),
            'c' => Some(Flavor::Charm),
            's' => Some(Flavor::Strange),
            't' => Some(Flavor::Top),
            'b' => Some(Flavor::Bottom),
            'g' => Some(Flavor::Graphic),
            'n' => Some(Flavor::Neutrino),
            'e' => Some(Flavor::Electron),
            'l' => Some(Flavor::Lattice),
            'm' => Some(Flavor::Muon),
            _ => None,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Token<'a> {
    Quark { flavor: Flavor, count: usize },
    Annihilator,
    Text(&'a str),
    LineBreak,
    Space,
}

pub struct Lexer<'a> {
    source: &'a str,
    chars: Peekable<CharIndices<'a>>,
    at_line_start: bool,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            chars: source.char_indices().peekable(),
            at_line_start: true,
        }
    }

    fn current_index(&mut self) -> usize {
        self.chars
            .peek()
            .map(|(idx, _)| *idx)
            .unwrap_or(self.source.len())
    }

    fn bump(&mut self) -> Option<(usize, char)> {
        self.chars.next()
    }

    fn skip_comment_line(&mut self) -> bool {
        if !self.at_line_start {
            return false; // don't skip comments in the middle of a line
        }

        let mut temp_chars = self.chars.clone();

        while let Some((_, c)) = temp_chars.peek() {
            if *c == ' ' || *c == '\t' {
                temp_chars.next();
            } else {
                break;
            }
        }

        if let Some((_, first_c)) = temp_chars.next() {
            if first_c == '#' {
                self.consume_until_newline();
                return true;
            } else if first_c == '/'
                && let Some((_, second_c)) = temp_chars.next()
                && second_c == '/'
            {
                self.consume_until_newline();
                return true;
            }
        }
        false
    }

    fn consume_until_newline(&mut self) {
        while let Some((_, c)) = self.chars.peek() {
            if *c == '\n' || *c == '\r' {
                break;
            }
            self.bump();
        }
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        // Before each token cut, check if the current line is a comment
        // If so, skip it and continue directly
        while self.skip_comment_line() {
            // After skipping the comment, we are directly before the line break.
            // If a line break follows, consume it to start fresh with the next line.
            if let Some(&(_, '\n')) = self.chars.peek() {
                self.bump();
            } else if let Some(&(_, '\r')) = self.chars.peek() {
                self.bump();
                if let Some(&(_, '\n')) = self.chars.peek() {
                    self.bump();
                }
            }
            self.at_line_start = true;
        }

        let (start_idx, c) = self.bump()?;

        self.at_line_start = c == '\n' || c == '\r';

        match c {
            '\n' => Some(Token::LineBreak),
            ' ' | '\t' | '\r' => {
                while let Some(&(_, next_c)) = self.chars.peek() {
                    if next_c == ' ' || next_c == '\t' || next_c == '\r' {
                        self.bump();
                    } else {
                        break;
                    }
                }
                Some(Token::Space)
            }
            '.' => match self.chars.peek() {
                Some(&(_, '.')) => {
                    self.bump();
                    Some(Token::Annihilator)
                }
                Some(&(_, next_c)) if next_c.is_alphabetic() => {
                    if let Some(flavor) = Flavor::from_char(next_c) {
                        let mut count = 0;
                        let mut valid_quark = true;
                        let temp_chars = self.chars.clone();

                        for (_, temp_c) in temp_chars {
                            if temp_c == next_c {
                                count += 1;
                            } else if temp_c == ' ' || temp_c == '\n' || temp_c == '\r' {
                                break;
                            } else {
                                valid_quark = false;
                                break;
                            }
                        }

                        if valid_quark && count > 0 {
                            for _ in 0..count {
                                self.bump();
                            }
                            if let Some(&(_, term_c)) = self.chars.peek()
                                && term_c == ' '
                            {
                                self.bump();
                            }
                            return Some(Token::Quark { flavor, count });
                        }
                    }
                    let end_idx = self.current_index();
                    Some(Token::Text(&self.source[start_idx..end_idx]))
                }
                _ => Some(Token::Text(&self.source[start_idx..=start_idx])),
            },
            _ => {
                while let Some(&(_, next_c)) = self.chars.peek() {
                    if next_c == '.'
                        || next_c == '\n'
                        || next_c == ' '
                        || next_c == '\t'
                        || next_c == '\r'
                    {
                        break;
                    }
                    self.bump();
                }
                let end_idx = self.current_index();
                Some(Token::Text(&self.source[start_idx..end_idx]))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognizes_a_quark_by_dot_letters_space() {
        let mut lexer = Lexer::new(".uu Heading");
        assert_eq!(
            lexer.next(),
            Some(Token::Quark {
                flavor: Flavor::Up,
                count: 2
            })
        );
        assert_eq!(lexer.next(), Some(Token::Text("Heading")));
        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn recognizes_the_annihilator() {
        let mut lexer = Lexer::new("..");
        assert_eq!(lexer.next(), Some(Token::Annihilator));
        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn skips_hash_and_slash_comment_lines() {
        let mut lexer = Lexer::new("# a comment\nkept");
        // the comment line and its trailing newline are swallowed entirely
        assert_eq!(lexer.next(), Some(Token::Text("kept")));
        assert_eq!(lexer.next(), None);
    }
}
