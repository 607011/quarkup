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
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            chars: source.char_indices().peekable(),
        }
    }

    fn current_index(&mut self) -> usize {
        self.chars
            .peek()
            .map(|(idx, _)| *idx)
            .unwrap_or(self.source.len())
    }

    fn bump(&mut self) {
        self.chars.next();
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let (start_idx, c) = self.chars.next()?;

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
                        let mut temp_chars = self.chars.clone();

                        while let Some((_, temp_c)) = temp_chars.next() {
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
                            if let Some(&(_, term_c)) = self.chars.peek() {
                                if term_c == ' ' {
                                    self.bump();
                                }
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
