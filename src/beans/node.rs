use super::tokens::*;
use std::rc::Rc;
use String as Id;

#[derive(Debug)]
pub enum Stmt {
    ExprStmt(Expr),
    If(Vec<(Expr, Vec<Stmt>)>, Vec<Stmt>),
    While(Expr, Vec<Stmt>),
    Block(Vec<Stmt>),
    Var(Id, Option<Expr>),
    FunDef(Id, Vec<Id>, Rc<Vec<Stmt>>),
    StructDef(Id, Vec<Id>),
    EnumDef(Id, Vec<(Id, Option<Expr>)>),
    Return(Expr),
    Break,
    Continue,
}

#[derive(Debug)]
pub enum Expr {
    Unary(TokenType, Box<Expr>),
    Binary(Box<Expr>, TokenType, Box<Expr>),
    Id(String),
    Call(Box<Expr>, Vec<Expr>),
    Get(Box<Expr>, Box<Expr>),
    Num(f64),
    Str(String),
    Bool(bool),
    Grouping(Box<Expr>),
    Assign(Box<Expr>, Box<Expr>),
    LambdaDef(Vec<String>, Vec<Stmt>),
    Nil,
}

impl Expr {
    pub fn new_from_tok(t: &Token) -> Expr {
        match t.get_type() {
            TokenType::Num => Expr::Num(t.as_f64()),
            TokenType::Str => Expr::Str(t.as_String()),
            TokenType::True => Expr::Bool(true),
            TokenType::False => Expr::Bool(false),
            TokenType::Identifier => Expr::Id(t.as_Id()),
            _ => panic!("Can't convert Token to Expr!"),
        }
    }
}
