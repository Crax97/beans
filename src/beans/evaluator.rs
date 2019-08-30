use super::node::Expr::*;
use super::node::Stmt::*;
use super::node::*;
use super::tokens::TokenType::*;
use std::cell::RefCell;
use std::rc::Rc;

use super::environments::*;
use std::borrow::Borrow;

pub enum StatementResult {
    Ok(Option<Value>),
    Return(Value),
    Break,
    Continue,
}

pub trait Evaluate<S, E> {
    fn execute_statement(&mut self, s: &Stmt) -> S;
    fn evaluate(&mut self, e: &Expr) -> E;
}

pub struct Evaluator {
    global: Rc<RefCell<Env>>,
    current: Rc<RefCell<Env>>,
}

impl Evaluator {
    pub fn new() -> Evaluator {
        let global = Rc::new(RefCell::new(Env::new()));
        Evaluator {
            global: global.clone(),
            current: global,
        }
    }

    pub fn evaluate_in_env(&mut self, stmts: &Vec<Stmt>, env: Env) -> StatementResult {
        let old = self.current.clone();
        self.current = Rc::new(RefCell::new(env));
        let result = self.exec_block(stmts);
        self.current = old;
        result
    }

    fn is_true(v: &Value) -> bool {
        match v {
            Value::Num(n) => *n != 0.0,
            Value::Str(s) => s.len() != 0,
            Value::Bool(b) => *b == true,
            Value::Callable(c) => c.arity() != 0,
            Value::Struct(c) => c.arity() != 0,
            Value::StructInstance(_) => true,
            Value::Enum(_, fields) => fields.len() != 0,
            Value::Nil => false,
        }
    }

    fn exec_if(
        &mut self,
        branches: &Vec<(Expr, Vec<Stmt>)>,
        else_block: &Vec<Stmt>,
    ) -> StatementResult {
        for branch in branches {
            let val = self.evaluate(&branch.0);
            if Evaluator::is_true(&val.get_value()) {
                return self.exec_block(&branch.1);
            }
        }
        self.exec_block(else_block)
    }

    fn exec_while(&mut self, cond: &Expr, block: &Vec<Stmt>) -> StatementResult {
        while Evaluator::is_true(&self.evaluate(&cond).get_value()) {
            match self.exec_block(block) {
                StatementResult::Ok(_) => {}
                StatementResult::Return(v) => return StatementResult::Return(v),
                StatementResult::Continue => continue,
                StatementResult::Break => break,
            }
        }

        StatementResult::Ok(None)
    }
    fn exec_block(&mut self, block: &Vec<Stmt>) -> StatementResult {
        for stmt in block.iter() {
            let result = self.execute_statement(stmt);
            match result {
                StatementResult::Ok(_) => {}
                StatementResult::Return(v) => return StatementResult::Return(v),
                StatementResult::Break => break,
                StatementResult::Continue => return result,
            }
        }
        StatementResult::Ok(None)
    }
    fn exec_var(&mut self, id: &String, initializer: &Option<Expr>) -> StatementResult {
        let mut value = Value::Nil;
        if let Some(expr) = initializer {
            value = self.evaluate(&expr).get_value();
        }
        let ret = value.clone();
        self.current
            .borrow_mut()
            .set(id.clone(), Symbol::new(value));
        StatementResult::Ok(Some(ret))
    }
    fn exec_fundef(
        &mut self,
        id: &String,
        params: &Vec<String>,
        block: &Rc<Vec<Stmt>>,
    ) -> StatementResult {
        let closure = Value::Callable(Rc::new(Box::new(Closure::new(
            block.clone(),
            self.current.clone(),
            params.to_vec(),
        ))));
        let ret = closure.clone();
        self.current
            .borrow_mut()
            .set(id.clone(), Symbol::new(closure));
        StatementResult::Ok(Some(ret))
    }

    fn exec_structdef(&mut self, name: &String, members: &Vec<String>) -> StatementResult {
        let strukt = Value::Struct(Rc::new(BaseStruct::new(members.to_vec(), name.clone())));
        let ret = strukt.clone();
        self.current
            .borrow_mut()
            .set(name.clone(), Symbol::new(strukt));
        StatementResult::Ok(Some(ret))
    }

    fn exec_enumdef(
        &mut self,
        name: &String,
        values: &Vec<(String, Option<Expr>)>,
    ) -> StatementResult {
        let mut i = 0.0;
        let mut variants: Vec<(String, f64)> = Vec::new();
        for value in values {
            let assoc_value = if let Some(expr) = &value.1 {
                match self.evaluate(&expr).get_value() {
                    Value::Num(e) => e,
                    _ => panic!("Enum variants can only be associated to numbers!"),
                }
            } else {
                let ii = i;
                i += 1.0;
                ii
            };
            variants.push((value.0.clone(), assoc_value));
        }
        let enumt = Value::Enum(name.clone(), variants);
        let ret = enumt.clone();
        self.current
            .borrow_mut()
            .set(name.clone(), Symbol::new(enumt));
        StatementResult::Ok(Some(ret))
    }

    fn exec_return(&mut self, e: &Expr) -> StatementResult {
        let v = self.evaluate(e);
        StatementResult::Return(v.get_value())
    }

    fn negate(v: Value) -> Value {
        match v {
            Value::Num(n) => Value::Num(-n),
            Value::Bool(b) => Value::Bool(!b),
            _ => panic!("Value can't be negated!"),
        }
    }

    fn arithmetic(&mut self, le: &Expr, op: super::tokens::TokenType, re: &Expr) -> Value {
        match op {
            Plus => {
                let l = self.evaluate(le).get_value();
                let r = self.evaluate(re).get_value();
                if l.is_numeric() && r.is_numeric() {
                    Value::Num(l.as_numeric() + r.as_numeric())
                } else {
                    return match (l, r) {
                        (Value::Str(ls), Value::Str(rs)) => Value::Str(format!("{}{}", ls, rs)),
                        (_, _) => panic!("Unsummable types!"),
                    };
                }
            }
            Minus => {
                let l = self.evaluate(le).get_value();
                let r = self.evaluate(re).get_value();
                Value::Num(l.as_numeric() - r.as_numeric())
            }
            Star => {
                let l = self.evaluate(le).get_value();
                let r = self.evaluate(re).get_value();
                Value::Num(l.as_numeric() * r.as_numeric())
            }
            Slash => {
                let l = self.evaluate(le).get_value();
                let r = self.evaluate(re).get_value();
                let divisor = r.as_numeric();
                Value::Num(if divisor != 0.0 {
                    l.as_numeric() / r.as_numeric()
                } else {
                    0.0
                })
            }
            Less => {
                let l = self.evaluate(le).get_value();
                let r = self.evaluate(re).get_value();
                Value::Bool(l.as_numeric() < r.as_numeric())
            }
            LessEquals => {
                let l = self.evaluate(le).get_value();
                let r = self.evaluate(re).get_value();
                Value::Bool(l.as_numeric() <= r.as_numeric())
            }
            More => {
                let l = self.evaluate(le).get_value();
                let r = self.evaluate(re).get_value();
                Value::Bool(l.as_numeric() > r.as_numeric())
            }
            MoreEquals => {
                let l = self.evaluate(le).get_value();
                let r = self.evaluate(re).get_value();
                Value::Bool(l.as_numeric() >= r.as_numeric())
            }
            EqualsEquals => {
                let l = self.evaluate(le).get_value();
                let r = self.evaluate(re).get_value();
                Value::Bool(l.as_numeric() == r.as_numeric())
            }
            BangEquals => {
                let l = self.evaluate(le).get_value();
                let r = self.evaluate(re).get_value();
                Value::Bool(l.as_numeric() != r.as_numeric())
            }

            LessLess => {
                let l = self.evaluate(le).get_value();
                let r = self.evaluate(re).get_value();
                Value::Num(((l.as_numeric() as u64) << (r.as_numeric() as u64)) as f64)
            }
            MoreMore => {
                let l = self.evaluate(le).get_value();
                let r = self.evaluate(re).get_value();
                Value::Num(((l.as_numeric() as u64) >> (r.as_numeric() as u64)) as f64)
            }
            Ampersand => {
                let l = self.evaluate(le).get_value();
                let r = self.evaluate(re).get_value();
                Value::Num(((l.as_numeric() as u64) & (r.as_numeric() as u64)) as f64)
            }
            Pipe => {
                let l = self.evaluate(le).get_value();
                let r = self.evaluate(re).get_value();
                Value::Num(((l.as_numeric() as u64) | (r.as_numeric() as u64)) as f64)
            }
            And => {
                let l = self.evaluate(le).get_value();
                if !Evaluator::is_true(&l) {
                    Value::Bool(false)
                } else {
                    self.evaluate(re).get_value()
                }
            }
            Or => {
                let l = self.evaluate(le).get_value();
                if Evaluator::is_true(&l) {
                    Value::Bool(true)
                } else {
                    self.evaluate(re).get_value()
                }
            }
            _ => unreachable!(),
        }
    }

    fn do_call(&mut self, fun: &Expr, args: &Vec<Expr>) -> Rc<Symbol> {
        let callable_maybe = self.evaluate(fun).get_value();
        let result = match callable_maybe {
            Value::Callable(call) => {
                if call.arity() != args.len() {
                    panic!("Arguments differ in size!");
                }

                let mut args_evaluated: Vec<Value> = Vec::new();
                for arg in args {
                    let evaluated = self.evaluate(arg).get_value();
                    args_evaluated.push(evaluated);
                }

                call.call(self, args_evaluated)
            }
            _ => panic!("Not callable!"),
        };
        Rc::new(Symbol::new(result))
    }

    fn get_symbol(&self, s: &String) -> Rc<Symbol> {
        self.current.as_ref().borrow().get(s)
    }
}

impl Evaluate<StatementResult, Rc<Symbol>> for Evaluator {
    fn execute_statement(&mut self, s: &Stmt) -> StatementResult {
        match s {
            Stmt::ExprStmt(e) => StatementResult::Ok(Some(self.evaluate(&e).get_value())),
            Stmt::If(branches, else_block) => self.exec_if(branches, else_block),
            Stmt::While(cond, block) => self.exec_while(cond, block),
            Stmt::Block(stmts) => self.exec_block(stmts),
            Stmt::Var(id, expr) => self.exec_var(id, expr),
            Stmt::FunDef(name, params, block) => self.exec_fundef(name, params, block),
            Stmt::StructDef(name, members) => self.exec_structdef(name, members),
            Stmt::EnumDef(name, values) => self.exec_enumdef(name, values),
            Stmt::Return(expr) => self.exec_return(expr),
            Stmt::Break => StatementResult::Break,
            Stmt::Continue => StatementResult::Continue,
        }
    }

    fn evaluate(&mut self, e: &Expr) -> Rc<Symbol> {
        match e {
            Expr::Num(n) => Rc::new(Symbol::new(Value::Num(*n))),
            Expr::Str(s) =>  Rc::new(Symbol::new(Value::Str(s.clone()))),
            Expr::Bool(b) =>  Rc::new(Symbol::new(Value::Bool(*b))),

            Unary(op, e) => Rc::new(Symbol::new(match op {
                Plus => self.evaluate(e).get_value(),
                Minus => Evaluator::negate(self.evaluate(e).get_value()),
                Bang => Evaluator::negate(self.evaluate(e).get_value()),
                _ => unreachable!(),
            })),
            Binary(l, op, r) => Rc::new(Symbol::new(self.arithmetic(l, *op, r))),
            Grouping(e) => self.evaluate(e),
            Id(name) => self.get_symbol(name),
            Call(exp, args) => self.do_call(exp, args),
            _ => panic!("Asd"),
        }
    }
}
