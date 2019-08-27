#[macro_use]
extern crate lazy_static;
mod beans;
fn main() {
    let test = String::from("2 + 3 - 6");
    let mut parser = beans::parser::Parser::new(beans::lexer::Lexer::new(test));
    let result = parser.parse();
}
