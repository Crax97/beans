import sys

top = """
use std::string::String;
use std::collections::HashMap;
#[derive(PartialEq, PartialOrd, Copy, Clone, Debug)]
pub enum TokenType { 
"""
bottom = """
}
"""

map_top = """
lazy_static! {
        pub static ref TOKENS_MAP : HashMap<String, TokenType> = {
        let mut m = HashMap::new();
"""
map_bottom = """
        m
    };
}"""

token_struct = """

#[derive(Debug)]
pub enum Value {
    Empty,
    Num(f64),
    Str(String),
    Id(String),
}

#[derive(Debug)]
pub struct Token {
    token_type : TokenType,
    line: u16,
    val: Value,
}

impl Token{
    pub fn new(token_type : TokenType, line : u16, val : Value) -> Token {
        Token {
            token_type : token_type,
            line : line,
            val : val,
        }
    }

    pub fn get_val(&self) -> &Value {
        &self.val
    }

    pub fn get_type(&self) -> TokenType {
        self.token_type
    }

    pub fn get_line(&self) -> u16 {
        self.line
    }

    pub fn as_f64(&self) -> f64 {
        match self.val {
            Value::Num(n) => n,
            _ => panic!("Falied converting Value to f64!")
        }
    }

    pub fn as_String(&self) -> String {
        match &self.val {
            Value::Str(s) => s.clone(),
            _ => panic!("Falied converting Value to String!")
        }
    }

    pub fn as_Id(&self) -> String {
        match &self.val {
            Value::Id(s) => s.clone(),
            _ => panic!("Falied converting Value to Id!")
        }
    }
}
"""

char_map = {
    '+': "Plus",
    '-': "Minus",
    '/': "Slash",
    '%': "Mod",
    '*': "Star",
    '=': "Equals",
    '<': "Less",
    '>': "More",
    '!': "Bang",
    '.': "Dot",
    ',': "Comma",
    ';': "Semicolon",
    ':': "Colon",
    '&': "Ampersand",
    '|': "Pipe",
    '?': "Question",
    '#': "Hashtag",
    '(': "LeftParen",
    ')': "RightParen",
    '{': "LeftBrace",
    '}': "RightBrace",
    '[': "LeftSquare",
    ']': "RightSquare",
}


def extract_token(file):
    token = ""
    generator = ""
    c = file.read(1)
    if c == '':
        raise "EOF while extracting token! was parsing " + token
    if not c.isprintable():
        raise "Found unprintable character! was parsing " + token
    while c != '"':
        if c in char_map:
            token += char_map[c]
        else:
            token = token + c
        generator += c
        c = file.read(1)
    token = token[0].upper() + token[1:]
    return (generator, token)


def main():
    if len(sys.argv) < 3:
        print("Usage:", sys.argv[0], "input_grammar output_file")

    input_file = open(sys.argv[1], "r")
    tokens = {("", "Eof"), ("", "Str"), ("", "Num"),
              ("", "Identifier")}
    c = input_file.read(1)
    while c != '':
        if c == '"':
            (generator, token) = extract_token(input_file)
            tokens = tokens | {(generator, token)}
        c = input_file.read(1)

    output_file = open(sys.argv[2], "w")
    output_file.write(top)
    for (_, token) in tokens:
        output_file.write(token + ",\n")
    output_file.write(bottom)

    output_file.write(map_top)
    for(gen, tok) in tokens:
        if(len(gen) > 0 and gen[0].isalpha()):
            template = '        m.insert(String::from("%s"), TokenType::%s);\n'
            output_file.write(template % (gen, tok))

    output_file.write(map_bottom)
    output_file.write(token_struct)


main()
