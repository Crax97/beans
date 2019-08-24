
use std::string::String;
use std::collections::HashMap;
#[derive(PartialEq, PartialOrd, Copy, Clone)]
pub enum TokenType { 
MoreEquals,
Enum,
Star,
Str,
Or,
Continue,
Fun,
RightParen,
Identifier,
In,
Return,
Else,
For,
Eof,
Not,
Minus,
Comma,
Dot,
RightBrace,
Less,
LessLess,
Lambda,
Var,
While,
Boolean,
Equals,
EqualsEquals,
And,
BangEquals,
Elif,
End,
Struct,
LeftParen,
LessEquals,
Slash,
Num,
Pass,
MoreMore,
If,
Then,
More,
Plus,
Do,
LeftBrace,
Break,

}

lazy_static! {
        pub static ref TOKENS_MAP : HashMap<String, TokenType> = {
        let mut m = HashMap::new();
        m.insert(String::from("enum"), TokenType::Enum);
        m.insert(String::from("or"), TokenType::Or);
        m.insert(String::from("continue"), TokenType::Continue);
        m.insert(String::from("fun"), TokenType::Fun);
        m.insert(String::from("in"), TokenType::In);
        m.insert(String::from("return"), TokenType::Return);
        m.insert(String::from("else"), TokenType::Else);
        m.insert(String::from("for"), TokenType::For);
        m.insert(String::from("not"), TokenType::Not);
        m.insert(String::from("lambda"), TokenType::Lambda);
        m.insert(String::from("var"), TokenType::Var);
        m.insert(String::from("while"), TokenType::While);
        m.insert(String::from("and"), TokenType::And);
        m.insert(String::from("elif"), TokenType::Elif);
        m.insert(String::from("end"), TokenType::End);
        m.insert(String::from("struct"), TokenType::Struct);
        m.insert(String::from("pass"), TokenType::Pass);
        m.insert(String::from("if"), TokenType::If);
        m.insert(String::from("then"), TokenType::Then);
        m.insert(String::from("do"), TokenType::Do);
        m.insert(String::from("break"), TokenType::Break);

        m
    };
}

pub enum Value {
    Empty,
    Num(f64),
    Str(String),
    Id(String),
}

pub struct Token {
    token_type : TokenType,
    line: u8,
    val: Value,
}

impl Token{
    pub fn new(token_type : TokenType, line : u8, val : Value) -> Token {
        Token {
            token_type : token_type,
            line : line,
            val : val,
        }
    }
}
