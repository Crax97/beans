use super::tokens::TokenType::*;
use super::tokens::TOKENS_MAP as TokenMap;
use super::tokens::*;

use super::reader::Reader;
use std::io::Read;

macro_rules! match_next {
    ( $c: expr, $f : expr, $ci: expr => $ti: expr, $( $cs: expr => $ts : expr ),*) => { {
        match $c {
                    $ci => { $ti },
                    $($cs => {$ts},)*
                    _ => { $f }
            }
    }};
}

#[cfg(test)]
mod tests {
    #[macro_use]
    use super::*;
    use super::super::tokens::TokenType::*;
    #[test]
    fn test_match_next() {
        assert!(
            match_next! { '<', Less,
                '<' => LessLess,
                '=' => LessEquals

            } == LessLess
        );
        assert!(
            match_next! { '=', Less,
                '!' => LessLess,
                '|' => LessEquals

            } == Less
        );
    }
}

macro_rules! peek_char {
    ($c : expr) => {
        if let Some(c) = $c.peek() {
            c
        } else {
            panic!("No char to peek at!");
        }
    };
}

pub struct Lexer {
    tokens: Vec<Token>,
    input_file: Reader,
    line: u8,
}

impl Lexer {
    fn new<T: std::io::Read>(input_file: &mut T) -> Lexer {
        Lexer {
            tokens: vec![],
            input_file: Reader::new(input_file),
            line: 0,
        }
    }

    fn do_lex(&mut self) {
        while let Some(c) = self.input_file.next() {
            match c {
                '\t' | ' ' => {
                    continue;
                }
                '\n' => {
                    self.line += 1;
                    continue;
                }
                _ => {}
            }
            let token = match c {
                '+' => Some(Plus),
                '-' => Some(Minus),
                '*' => Some(Star),
                '/' => Some(Slash),
                '#' => {
                    self.comment();
                    continue;
                }
                '(' => Some(LeftParen),
                ')' => Some(RightParen),
                '{' => Some(LeftBrace),
                '}' => Some(RightBrace),
                '<' => Some(match_next! { peek_char!(self.input_file),
                    Less,
                    '<' => LessLess,
                    '=' => LessEquals
                }),
                '>' => Some(match_next! { peek_char!(self.input_file),
                    More,
                    '>' => MoreMore,
                    '=' => MoreEquals
                }),
                '=' => Some(match_next! {  peek_char!(self.input_file),
                    Equals,
                    '=' => EqualsEquals,
                }),
                ',' => Some(Comma),
                '.' => Some(Dot),
                _ => None,
            };

            if let Some(tok) = token {
                self.tokens.push(Token::new(tok, self.line, Value::Empty));
                continue;
            }

            let string = match c {
                '"' | '\'' => Some(self.string(c)),
                _ => None,
            };
            if let Some(s) = string {
                self.tokens.push(Token::new(Str, self.line, Value::Str(s)));
                continue;
            }

            if c.is_digit(10) {
                let n = self.num();
                self.tokens.push(Token::new(Num, self.line, Value::Num(n)));
                continue;
            }

            if c.is_alphabetic() {
                let id = self.id();
                if TokenMap.contains_key(&id) {
                    self.tokens.push(Token::new(
                        *TokenMap.get(&id).unwrap(),
                        self.line,
                        Value::Empty,
                    ));
                } else {
                    self.tokens
                        .push(Token::new(Identifier, self.line, Value::Empty))
                }
            }
        }
    }
    fn comment(&mut self) {
        while let Some(c) = self.input_file.peek() {
            if c == '\n' {
                break;
            }
            self.input_file.next();
            continue;
        }
    }

    fn string(&mut self, end: char) -> String {
        let mut s = String::new();
        while let Some(c) = self.input_file.peek() {
            self.input_file.next();
            if c == end {
                break;
            }
            if c == '\n' {
                self.line += 1;
            }
            s.push(c);
        }
        s
    }

    fn num(&mut self) -> f64 {
        let prev = self.input_file.prev().unwrap();
        let mut s = char::to_string(&prev);
        while let Some(c) = self.input_file.peek() {
            self.input_file.next();
            if !(c.is_digit(10) || c == '.') {
                break;
            }
            s.push(c);
        }
        if let Ok(n) = str::parse(s.as_ref()) {
            return n;
        } else {
            panic!(format!("Faliure parsing number, at line ${}!", self.line))
        }
    }

    fn id(&mut self) -> String {
        let prev = self.input_file.prev().unwrap();
        let mut s = char::to_string(&prev);
        while let Some(c) = self.input_file.peek() {
            self.input_file.next();
            if !(c.is_alphanumeric() || c == '_') {
                break;
            }
            s.push(c);
        }
        s
    }
}
