use super::node::Expr::*;
use super::node::Stmt::*;
use super::node::*;
use super::tokens::Token;
use super::tokens::TokenType::*;
use float_cmp::*;
use std::cell::RefCell;
use std::rc::Rc;

use super::environments::*;

#[cfg(test)]
mod tests {
    use super::super::evaluator::Evaluate;
    use super::super::evaluator::Evaluator;
    use super::super::evaluator::StatementResult;
    use super::super::*;
    #[test]
    fn try_simple_expr() {
        let expr = "3 * 2;";
        let lexer = lexer::Lexer::new(String::from(expr));
        let mut parser = parser::Parser::new(lexer);
        let stmts = parser.parse();

        let mut evaluator = evaluator::Evaluator::new();
        for stmt in stmts {
            match evaluator.execute_statement(&stmt) {
                StatementResult::Ok(sv) => {
                    println!("{}", sv.stringfiy());
                }
                _ => {}
            }
        }
    }

    fn exec_single_str(s: &str) -> StatementResult {
        let lexer = lexer::Lexer::new(String::from(s));
        let mut parser = parser::Parser::new(lexer);
        let stmts = parser.parse();

        let mut evaluator = evaluator::Evaluator::new();
        return evaluator.execute_statement(&stmts.first().unwrap());
    }

    fn exec_prog(s: &str) -> StatementResult {
        let lexer = lexer::Lexer::new(String::from(s));
        let mut parser = parser::Parser::new(lexer);
        let stmts = parser.parse();

        let mut evaluator = evaluator::Evaluator::new();
        let mut value: StatementResult = StatementResult::Break;
        for stmt in stmts {
            value = evaluator.execute_statement(&stmt);
        }
        value
    }

    #[test]
    fn try_logical() {
        match exec_single_str("2 == 2;") {
            StatementResult::Ok(v) => assert!(v.as_numeric() == 1.0),
            _ => panic!("Wrong result in 2 == 2!"),
        }
        match exec_single_str("2 != 2;") {
            StatementResult::Ok(v) => assert!(v.as_numeric() != 1.0),
            _ => panic!("Wrong result in 2 != 2!"),
        }
        match exec_single_str("2 < 3;") {
            StatementResult::Ok(v) => assert!(v.as_numeric() == 1.0),
            _ => panic!("Wrong result in 2 < 3!"),
        }
        match exec_single_str("2 > 3;") {
            StatementResult::Ok(v) => assert!(v.as_numeric() != 1.0),
            _ => panic!("Wrong result in 2 > 3!"),
        }
        match exec_single_str("(2 > 3) or (3 > 1);") {
            StatementResult::Ok(v) => assert!(v.as_numeric() == 1.0),
            _ => panic!("Wrong result in 2 > 3!"),
        }
        match exec_single_str("(2 > 3) and (3 > 1);") {
            StatementResult::Ok(v) => assert!(v.as_numeric() != 1.0),
            _ => panic!("Wrong result in 2 > 3!"),
        }
        match exec_single_str("(2 > 3) or (3 > 1);") {
            StatementResult::Ok(v) => assert!(v.as_numeric() == 1.0),
            _ => panic!("Wrong result in 2 > 3!"),
        }
        match exec_single_str("((2 > 3) and (3 > 1)) or (2 + 3 - 5 * 6 < 0);") {
            StatementResult::Ok(v) => assert!(v.as_numeric() == 1.0),
            _ => panic!("Wrong result in 2 > 3!"),
        }
        match exec_single_str("((2 > 3) and (3 > 1)) and (2 + 3 - 5 * 6 < 0);") {
            StatementResult::Ok(v) => assert!(v.as_numeric() != 1.0),
            _ => panic!("Wrong result in 2 > 3!"),
        }
    }

    #[test]
    fn try_statements() {
        let if_example = "if 2 < 3 then 
                return 3.14;
            else
                return 6.28;
            end";
        match exec_single_str(if_example) {
            StatementResult::Return(val) => assert!(val.as_numeric() == 3.14),
            _ => panic!("Return failed"),
        }

        match exec_single_str(
            "if 2 < 3 then 
                    if true then
                        return 3.14;
                    end
            else
                return 6.28;
            end",
        ) {
            StatementResult::Return(val) => assert!(val.as_numeric() == 3.14),
            _ => panic!("Return failed"),
        }

        match exec_prog(
            "function return_1()
                return 1;
            end
            return_1();",
        ) {
            StatementResult::Ok(val) => assert!(val.as_numeric() == 1.0),
            _ => panic!("Ok 1 failed"),
        }

        match exec_prog(
            "function return_from_if() 
                if true then
                    return 3.14;
                end
            end
            return_from_if();",
        ) {
            StatementResult::Ok(val) => assert!(val.as_numeric() == 3.14),
            _ => panic!("Ok 2 failed"),
        }

        match exec_prog(
            "function recursive(n) 
                if n == 1 then
                    return 3.14;
                else
                    return recursive(1);
                end
            end
            recursive(10);",
        ) {
            StatementResult::Ok(val) => assert!(val.as_numeric() == 3.14),
            _ => panic!("Ok 3 failed"),
        }
    }

    #[test]
    fn boolean() {
        let factorial_prog = "function boolean(n)
                if n == 3.14 or n == 6 then
                    return true;
                end
                return false;
            end
            boolean(3.14);";
        match exec_prog(factorial_prog) {
            StatementResult::Ok(v) => assert!(v.as_numeric() == 1.0),
            _ => panic!("Failure on factorial(5)"),
        }
    }
    #[test]
    fn factorial() {
        let factorial_prog = "function factorial(n)
                if n == 1 or n == 0 then
                    return 1;
                end
                return n * factorial(n - 1);
            end
            factorial(5);";
        match exec_prog(factorial_prog) {
            StatementResult::Ok(v) => assert!(v.as_numeric() == 120.0),
            _ => panic!("Failure on factorial(5)"),
        }
    }

    #[test]
    fn assign() {
        let prog = "var pi = 3.14;
        pi;";
        match exec_prog(prog) {
            StatementResult::Ok(v) => assert!(v.as_numeric() == 3.14),
            _ => panic!("Failure on factorial(5)"),
        }
    }
}

pub enum StatementResult {
    Ok(Value),
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
        Evaluator::new_with_global(Rc::new(RefCell::new(Env::new())))
    }

    pub fn new_with_global(env: Rc<RefCell<Env>>) -> Evaluator {
        Evaluator {
            global: env.clone(),
            current: env.clone(),
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
            Value::Collection(map) => map.len() != 0,
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

        StatementResult::Ok(Value::Nil)
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
        StatementResult::Ok(Value::Nil)
    }
    fn exec_var(&mut self, id: &String, initializer: &Option<Expr>) -> StatementResult {
        let mut value = Value::Nil;
        if let Some(expr) = initializer {
            value = self.evaluate(&expr);
        }
        let ret = value.clone();
        self.current.borrow_mut().set(id.clone(), value.clone());
        StatementResult::Ok(ret)
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
        self.current.borrow_mut().set(id.clone(), closure);
        StatementResult::Ok(ret)
    }

    fn exec_structdef(&mut self, name: &String, members: &Vec<String>) -> StatementResult {
        let strukt = Value::Struct(Rc::new(BaseStruct::new(members.to_vec(), name.clone())));
        let ret = strukt.clone();
        self.current.borrow_mut().set(name.clone(), strukt);
        StatementResult::Ok(ret)
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
        let ret = enumt.clone();
        self.current.borrow_mut().set(name.clone(), enumt);
        StatementResult::Ok(ret)
    }

    fn exec_return(&mut self, e: &Expr) -> StatementResult {
        let v = self.evaluate(e);
        StatementResult::Return(v)
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
                let l = self.evaluate(le);
                let r = self.evaluate(re);
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
                let l = self.evaluate(le);
                let r = self.evaluate(re);
                Value::Num(l.as_numeric() - r.as_numeric())
            }
            Star => {
                let l = self.evaluate(le);
                let r = self.evaluate(re);
                Value::Num(l.as_numeric() * r.as_numeric())
            }
            Slash => {
                let l = self.evaluate(le);
                let r = self.evaluate(re);
                let divisor = r.as_numeric();
                Value::Num(if divisor != 0.0 {
                    l.as_numeric() / r.as_numeric()
                } else {
                    0.0
                })
            }
            Less => {
                let l = self.evaluate(le);
                let r = self.evaluate(re);
                Value::Bool(l.as_numeric() < r.as_numeric())
            }
            LessEquals => {
                let l = self.evaluate(le);
                let r = self.evaluate(re);
                Value::Bool(l.as_numeric() <= r.as_numeric())
            }
            More => {
                let l = self.evaluate(le);
                let r = self.evaluate(re);
                Value::Bool(l.as_numeric() > r.as_numeric())
            }
            MoreEquals => {
                let l = self.evaluate(le);
                let r = self.evaluate(re);
                Value::Bool(l.as_numeric() >= r.as_numeric())
            }
            EqualsEquals => {
                let l = self.evaluate(le);
                let r = self.evaluate(re);
                let ln = l.as_numeric();
                let rn = r.as_numeric();
                Value::Bool(ln.approx_eq(rn, F64Margin::default()))
            }
            BangEquals => {
                let l = self.evaluate(le);
                let r = self.evaluate(re);
                Value::Bool(l.as_numeric() != r.as_numeric())
            }

            LessLess => {
                let l = self.evaluate(le);
                let r = self.evaluate(re);
                Value::Num(((l.as_numeric() as u64) << (r.as_numeric() as u64)) as f64)
            }
            MoreMore => {
                let l = self.evaluate(le);
                let r = self.evaluate(re);
                Value::Num(((l.as_numeric() as u64) >> (r.as_numeric() as u64)) as f64)
            }
            Ampersand => {
                let l = self.evaluate(le);
                let r = self.evaluate(re);
                Value::Num(((l.as_numeric() as u64) & (r.as_numeric() as u64)) as f64)
            }
            Pipe => {
                let l = self.evaluate(le);
                let r = self.evaluate(re);
                Value::Num(((l.as_numeric() as u64) | (r.as_numeric() as u64)) as f64)
            }
            And => {
                let l = self.evaluate(le);
                if !Evaluator::is_true(&l) {
                    Value::Bool(false)
                } else {
                    self.evaluate(re)
                }
            }
            Or => {
                let l = self.evaluate(le);
                if Evaluator::is_true(&l) {
                    Value::Bool(true)
                } else {
                    self.evaluate(re)
                }
            }
            _ => unreachable!(),
        }
    }

    fn do_call(&mut self, fun: &Expr, args: &Vec<Expr>) -> Value {
        let callable_maybe = self.evaluate(fun);

        match callable_maybe {
            Value::Callable(call) => {
                if call.arity() != args.len() {
                    panic!("Arguments differ in size!");
                }

                let mut args_evaluated: Vec<Value> = Vec::new();
                for arg in args {
                    let evaluated = self.evaluate(arg);
                    args_evaluated.push(evaluated);
                }

                call.call(self, args_evaluated)
            }
            _ => Value::Nil,
        }
    }

    fn get(&mut self, l: &Expr, id: &String) -> Value {
        let base = self.evaluate(l);
        match base {
            Value::Collection(map) => match map.get(id) {
                Some(val) => val.clone(),
                None => Value::Nil,
            },
            _ => panic!("Can't get from values different from collections!"),
        }
    }
    fn assign(&mut self, l: &Expr, r: &Expr) -> Value {
        let value = self.evaluate(r);
        match l {
            Expr::Id(name) => {
                let mut current_env = self.current.as_ref().borrow_mut();
                current_env.set(name.clone(), value.clone());
            }
            Get(expr, id) => {
                let base = self.evaluate(expr);
                match base {
                    Value::Collection(mut map) => {
                        map.insert(id.clone(), value.clone());
                    }
                    _ => {
                        eprint!("Invalid assign target!");
                    }
                }
            }
            _ => {
                eprint!("Invalid assign target!");
            }
        }
        value
    }
    fn lambda(&mut self, params: &Vec<String>, prog: Rc<Vec<Stmt>>) -> Value {
        Value::Callable(Rc::new(Box::new(Closure::new(
            prog,
            self.current.clone(),
            params.clone(),
        ))))
    }

    fn get_value(&self, id: &String) -> Value {
        let current_env = self.current.as_ref().borrow();
        current_env.get(id).clone()
    }
}

impl Evaluate<StatementResult, Value> for Evaluator {
    fn execute_statement(&mut self, s: &Stmt) -> StatementResult {
        match s {
            Stmt::ExprStmt(e) => StatementResult::Ok(self.evaluate(&e)),
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

    fn evaluate(&mut self, e: &Expr) -> Value {
        match e {
            Expr::Num(n) => Value::Num(*n),
            Expr::Str(s) => Value::Str(s.clone()),
            Expr::Bool(b) => Value::Bool(*b),

            Unary(op, e) => match op {
                Plus => self.evaluate(e),
                Minus => Evaluator::negate(self.evaluate(e)),
                Not => Evaluator::negate(self.evaluate(e)),
                _ => unreachable!(),
            },
            Binary(l, op, r) => self.arithmetic(l, *op, r),
            Grouping(e) => self.evaluate(e),
            Id(name) => self.get_value(name),
            Call(exp, args) => self.do_call(exp, args),
            Get(l, r) => self.get(l, r),
            Assign(l, r) => self.assign(l, r),
            LambdaDef(params, stmts) => self.lambda(params, stmts.clone()),
            Nil => Value::Nil,
        }
    }
}
