use super::tokens::TokenType::*;
use super::tokens::TOKENS_MAP as TokenMap;
use super::tokens::*;

use super::reader::Reader;
use std::io::Read;

macro_rules! peek_char {
    ($c : expr) => {
        if let Some(c) = $c.peek() {
            c
        } else {
            panic!("No char to peek at!");
        }
    };
}

macro_rules! match_next {
    ( $c: expr, $f : expr, $ci: expr => $ti: expr, $( $cs: expr => $ts : expr ),*) => { {
        match peek_char!($c) {
                    $ci => {
                        $c.next().unwrap(); $ti },
                    $($cs => {$c.next().unwrap();$ts},)*
                    _ => { $f }
            }
    }};
}

#[cfg(test)]
mod tests {
    #[macro_use]
    use super::*;
    use super::super::tokens::TokenType::*;
    // #[test]
    // fn test_match_next() {
    //     assert!(
    //         match_next! { Some('<'), Less,
    //             '<' => LessLess,
    //             '=' => LessEquals

    //         } == LessLess
    //     );
    //     assert!(
    //         match_next! { Some('='), Less,
    //             '!' => LessLess,
    //             '|' => LessEquals

    //         } == Less
    //     );
    // }

    #[test]
    fn test_lexing_ok() {
        let test = String::from(
            "function hello(a, b, c) 
            return a + b - c * 3.14 << 3;
            end",
        );
        let mut lexer = Lexer::new(test);
        lexer.do_lex();
        assert!(!lexer.is_at_end());
        assert!(lexer.peek().unwrap().get_type() == Function);
        assert!(lexer.next().unwrap().get_type() == Function);
        assert!(lexer.next().unwrap().get_type() == Identifier);
        assert!(lexer.next().unwrap().get_type() == LeftParen);
        assert!(lexer.next().unwrap().get_type() == Identifier);
        assert!(lexer.next().unwrap().get_type() == Comma);
        assert!(lexer.next().unwrap().get_type() == Identifier);
        assert!(lexer.next().unwrap().get_type() == Comma);
        assert!(lexer.next().unwrap().get_type() == Identifier);
        assert!(lexer.next().unwrap().get_type() == RightParen);
        assert!(lexer.next().unwrap().get_type() == Return);
        assert!(lexer.next().unwrap().get_type() == Identifier);
        assert!(lexer.prev().unwrap().get_type() == Identifier);
        assert!(lexer.peek().unwrap().get_type() == Plus);
        assert!(lexer.next().unwrap().get_type() == Plus);
        assert!(!lexer.is_at_end());
        assert!(lexer.next().unwrap().get_type() == Identifier);
        assert!(lexer.next().unwrap().get_type() == Minus);
        assert!(lexer.next().unwrap().get_type() == Identifier);
        assert!(lexer.next().unwrap().get_type() == Star);
        assert!(lexer.next().unwrap().get_type() == Num);
        assert!(lexer.next().unwrap().get_type() == LessLess);
        assert!(lexer.next().unwrap().get_type() == Num);
        assert!(lexer.next().unwrap().get_type() == Semicolon);
        assert!(lexer.prev().unwrap().get_type() == Semicolon);
        assert!(lexer.peek().unwrap().get_type() == End);
        assert!(lexer.next().unwrap().get_type() == End);

        assert!(lexer.is_at_end());
    }
}

pub struct Lexer {
    tokens: Vec<Token>,
    input_text: Reader,
    line: u16,
    cur_tok: usize,
}

impl Lexer {
    pub fn new(input_text: String) -> Lexer {
        Lexer {
            tokens: vec![],
            input_text: Reader::new(input_text),
            line: 1,
            cur_tok: 0,
        }
    }

    pub fn get_line(&self) -> u16 {
        self.line
    }

    pub fn next(&mut self) -> Option<&Token> {
        self.cur_tok += 1;
        return self.tokens.get(self.cur_tok - 1);
    }

    pub fn peek(&self) -> Option<&Token> {
        return self.tokens.get(self.cur_tok);
    }

    pub fn prev(&self) -> Option<&Token> {
        if self.cur_tok == 0 {
            return None;
        }
        return self.tokens.get(self.cur_tok - 1);
    }

    pub fn do_lex(&mut self) {
        while let Some(c) = self.input_text.next() {
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
                '<' => Some(match_next! { self.input_text,
                    Less,
                    '<' => LessLess,
                    '=' => LessEquals
                }),
                '>' => Some(match_next! { self.input_text,
                    More,
                    '>' => MoreMore,
                    '=' => MoreEquals
                }),
                '=' => Some(match_next! {  self.input_text,
                    Equals,
                    '=' => EqualsEquals,
                }),
                '!' => Some(match_next! {  self.input_text,
                    Not,
                    '=' => BangEquals,
                }),
                ',' => Some(Comma),
                '.' => Some(Dot),
                ';' => Some(Semicolon),
                ':' => Some(Colon),
                '[' => Some(LeftSquare),
                ']' => Some(RightSquare),
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

            let id = self.id();
            if c.is_alphabetic() {
                if TokenMap.contains_key(&id) {
                    self.tokens.push(Token::new(
                        *TokenMap.get(&id).unwrap(),
                        self.line,
                        Value::Id(id),
                    ));
                } else {
                    self.tokens
                        .push(Token::new(Identifier, self.line, Value::Id(id)))
                }
                continue;
            }

            panic!(format!("Unrecognized token at line {}: {}", self.line, id));
        }
    }

    pub fn is_at_end(&self) -> bool {
        self.cur_tok == self.tokens.len()
    }
    fn comment(&mut self) {
        while let Some(c) = self.input_text.peek() {
            if c == '\n' {
                break;
            }
            self.input_text.next();
            continue;
        }
    }

    fn string(&mut self, end: char) -> String {
        let mut s = String::new();
        while let Some(c) = self.input_text.next() {
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
        let prev = self.input_text.prev().unwrap();
        let mut s = String::new();
        s.push(prev);
        while let Some(c) = self.input_text.peek() {
            if !(c.is_numeric() || c == '.') {
                break;
            }
            self.input_text.next();
            s.push(c);
        }
        if let Ok(n) = str::parse::<f64>(s.as_ref()) {
            return n;
        } else {
            panic!(format!(
                "Faliure parsing number, at line {}! Got {}",
                self.line, s
            ))
        }
    }

    fn id(&mut self) -> String {
        let prev = self.input_text.prev().unwrap();
        let mut s = char::to_string(&prev);
        while let Some(c) = self.input_text.peek() {
            if !(c.is_alphanumeric() || c == '_') {
                break;
            }
            self.input_text.next();
            s.push(c);
        }
        s
    }
}
