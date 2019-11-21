use super::tokens::TokenType::*;
use super::tokens::TOKENS_MAP as TokenMap;
use super::tokens::*;

use super::reader::Reader;

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
    had_error: bool,
}

impl Lexer {
    pub fn new(input_text: String) -> Lexer {
        Lexer {
            tokens: vec![],
            input_text: Reader::new(input_text),
            line: 1,
            cur_tok: 0,
            had_error: false,
        }
    }

    pub fn had_error(&self) -> bool {
        self.had_error
    }

    pub fn get_line(&self) -> u16 {
        self.line
    }

    pub fn next(&mut self) -> Option<&Token> {
        self.lex_next();
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

    fn lex_next(&mut self) {
        if self.had_error {
            return;
        }
        let mut c: char = ' ';
        while let Some(n) = self.input_text.next() {
            if n == '\n' {
                self.line += 1;
            } else if n == '#' {
                self.comment();
            } else if !"\r\t ".contains(n) {
                c = n;
                break;
            }
        }

        if c == ' ' {
            return;
        }

        let token = match c {
            '+' => Some(Plus),
            '-' => Some(Minus),
            '*' => Some(Star),
            '/' => Some(Slash),
            '%' => Some(Mod),
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
            return;
        }
        if "\'\"".contains(c) {
            let s = self.string(c);
            self.tokens.push(Token::new(Str, self.line, Value::Str(s)));
            return;
        }

        if c.is_digit(10) {
            let n = self.num();
            self.tokens.push(Token::new(Num, self.line, Value::Num(n)));
            return;
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
            return;
        }

        self.tokens.push(Token::new(Eof, self.line, Value::Empty));
        self.had_error = true;
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
            if c == '\\' {
                let next = self.input_text.next();
                let next = match next {
                    Some(c) => c,
                    None => {
                        println!(
                            "Got EOF while parsing escape sequence at line {}!",
                            self.line
                        );
                        ' '
                    }
                };

                let escape_code: u8 = match next {
                    'a' => 0x07,  // \a
                    'b' => 0x08,  // \b
                    'e' => 0x1B,  // \e
                    'f' => 0x0C,  // \f
                    'n' => 0x0A,  // \n
                    'r' => 0x0D,  // \r
                    't' => 0x09,  // \t
                    'v' => 0x0B,  // \v
                    '\\' => 0x5C, // \
                    '\'' => 0x27, // '
                    '"' => 0x22,  // "
                    '?' => 0x3F,  // ?,
                    _ => {
                        println!(
                            "Unrecognised escape sequence at line {}: {}",
                            self.line,
                            format!("{}{}", c, next)
                        );
                        0x00
                    }
                };
                if escape_code != 0x00 {
                    s.push(escape_code as char)
                }
            } else {
                s.push(c);
            }
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
            println!("Faliure parsing number, at line {}! Got {}", self.line, s);
            self.had_error = true;
            0.0
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
