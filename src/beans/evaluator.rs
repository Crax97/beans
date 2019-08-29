use super::node::Expr::*;
use super::node::Stmt::*;
use super::node::*;
use std::cell::RefCell;
use std::rc::Rc;

use super::environments::*;

pub enum StatementResult {
    Ok(Option<Rc<Value>>),
    Return(Value),
    Break,
    Continue,
}

pub trait Evaluate<T> {
    fn execute_statement(&mut self, s: &Stmt) -> T;
    fn evaluate(&mut self, e: &Expr) -> Value;
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
            if Evaluator::is_true(&val) {
                return self.exec_block(&branch.1);
            }
        }
        self.exec_block(else_block)
    }

    fn exec_while(&mut self, cond: &Expr, block: &Vec<Stmt>) -> StatementResult {
        while Evaluator::is_true(&self.evaluate(&cond)) {
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
            value = self.evaluate(&expr);
        }
        let ret = Rc::new(value);
        self.current
            .borrow_mut()
            .set(id.clone(), Symbol::new(ret.clone()));
        StatementResult::Ok(Some(ret))
    }
    fn exec_fundef(
        &mut self,
        id: &String,
        params: &Vec<String>,
        block: &Rc<Vec<Stmt>>,
    ) -> StatementResult {
        let closure = Value::Callable(Box::new(Closure::new(
            block.clone(),
            self.current.clone(),
            params.to_vec(),
        )));
        let ret = Rc::new(closure);
        self.current
            .borrow_mut()
            .set(id.clone(), Symbol::new(ret.clone()));
        StatementResult::Ok(Some(ret))
    }

    fn exec_structdef(&mut self, name: &String, members: &Vec<String>) -> StatementResult {
        let strukt = Value::Struct(BaseStruct::new(members.to_vec(), name.clone()));
        let strukt_ref = Rc::new(strukt);
        self.current
            .borrow_mut()
            .set(name.clone(), Symbol::new(strukt_ref.clone()));
        StatementResult::Ok(Some(strukt_ref))
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
                match self.evaluate(&expr) {
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
        let enumt_ref = Rc::new(enumt);
        self.current
            .borrow_mut()
            .set(name.clone(), Symbol::new(enumt_ref.clone()));
        StatementResult::Ok(Some(enumt_ref))
    }

    fn exec_return(&mut self, e: &Expr) -> StatementResult {
        let v = self.evaluate(e);
        StatementResult::Return(v)
    }
}

impl Evaluate<StatementResult> for Evaluator {
    fn execute_statement(&mut self, s: &Stmt) -> StatementResult {
        match s {
            ExprStmt(e) => StatementResult::Ok(Some(Rc::new(self.evaluate(&e)))),
            If(branches, else_block) => self.exec_if(branches, else_block),
            While(cond, block) => self.exec_while(cond, block),
            Block(stmts) => self.exec_block(stmts),
            Var(id, expr) => self.exec_var(id, expr),
            FunDef(name, params, block) => self.exec_fundef(name, params, block),
            StructDef(name, members) => self.exec_structdef(name, members),
            EnumDef(name, values) => self.exec_enumdef(name, values),
            Return(expr) => self.exec_return(expr),
            Break => StatementResult::Break,
            Continue => StatementResult::Continue,
        }
    }

    fn evaluate(&mut self, e: &Expr) -> Value {
        match e {
            _ => Value::Bool(false),
        }
    }
}
