use super::node::Expr::*;
use super::node::*;
use super::tokens::TokenType::*;
use float_cmp::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use super::environments::*;

#[cfg(test)]
mod tests {
    use super::super::evaluator::Evaluate;
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

    #[test]
    fn dict() {
        let prog = "var dic = {a: 3.14, b: 2, k: 6.28};
        dic.k;";
        match exec_prog(prog) {
            StatementResult::Ok(v) => assert!(v.as_numeric() == 6.28),
            StatementResult::Failure(why) => panic!(format!("Failure! {}", why)),
            _ => panic!("Failure on dic.c"),
        }
    }

    #[test]
    fn list() {
        let prog = "var lis = [1, 2, 3, 4, 42];
        lis[4];";
        match exec_prog(prog) {
            StatementResult::Ok(v) => assert!(v.as_numeric() == 42.0),
            StatementResult::Failure(why) => panic!(format!("Failure! {}", why)),
            _ => panic!("Failure on lis! Got {:?}", prog),
        }
    }
    #[test]
    fn strukt() {
        let prog = "struct Point {x, y}
        var p = Point(10, 23);
        p.x;";
        match exec_prog(prog) {
            StatementResult::Ok(v) => assert!(v.as_numeric() == 10.0),
            StatementResult::Failure(why) => panic!(format!("Failure! {}", why)),
            _ => panic!("Failure on structs! Got {:?}", prog),
        }
    }
    #[test]
    fn enums() {
        let prog = "enum Colors {Red, Blue = 20}
        var c = Colors.Blue;
        c;";
        match exec_prog(prog) {
            StatementResult::Ok(v) => assert!(v.as_numeric() == 20.0),
            StatementResult::Failure(why) => panic!(format!("Failure! {}", why)),
            _ => panic!("Failure on enums! Got {:?}", prog),
        }
    }
}
macro_rules! operation {
    ($lr: expr, $op: tt, $rr: expr, $variant: ident) => {
        match ($lr, $rr) {
            (Ok(l), Ok(r)) => {
                if l.is_numeric() && r.is_numeric() {
                    Ok(Value::$variant(l.as_numeric() $op r.as_numeric()))
                } else {
                    Err(format!("Unsummable values! {}, {}", l.stringfiy(), r.stringfiy()))
                }
            },
            (Err(lwhy), _) => Err(format!("In left side of operation: {}", lwhy)),
            (_, Err(rwhy)) => Err(format!("In right side of operation: {}", rwhy)),
            }
        };
}

macro_rules! get_value {
    ($er: expr) => {
        match $er {
            Ok(l) => l,
            Err(lwhy) => return Err(lwhy),
        }
    };
}

macro_rules! get_values_no_bs {
    ($lr: expr, $rr: expr) => {
        match ($lr, $rr) {
            (Ok(l), Ok(r)) => (l, r),
            (Err(lwhy), _) => return Err(format!("In left side of operation: {}", lwhy)),
            (_, Err(rwhy)) => return Err(format!("In right side of operation: {}", rwhy)),
        }
    };
}

pub enum StatementResult {
    Ok(Value),
    Return(Value),
    Break,
    Continue,
    Failure(String),
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
            Value::StructInstance(_) => true,
            Value::Enum(_, fields) => fields.len() != 0,
            Value::Nil => false,
            Value::Collection(map) => map.len() != 0,
            Value::List(elts) => elts.len() != 0,
        }
    }

    fn exec_import(&mut self, module: &String) -> StatementResult {
        /// TODO: Add module importing
        println!("Importing {}", module);
        StatementResult::Ok(Value::Nil)
    }

    fn exec_if(
        &mut self,
        branches: &Vec<(Expr, Vec<Stmt>)>,
        else_block: &Vec<Stmt>,
    ) -> StatementResult {
        for branch in branches {
            if {
                let res = match self.evaluate(&branch.0) {
                    Ok(v) => v,
                    Err(why) => return StatementResult::Failure(why),
                };
                Evaluator::is_true(&res)
            } {
                return self.exec_block(&branch.1);
            }
        }
        self.exec_block(else_block)
    }

    fn exec_while(&mut self, cond: &Expr, block: &Vec<Stmt>) -> StatementResult {
        while {
            let res = match self.evaluate(&cond) {
                Ok(v) => v,
                Err(why) => return StatementResult::Failure(why),
            };
            Evaluator::is_true(&res)
        } {
            match self.exec_block(block) {
                StatementResult::Ok(_) => {}
                StatementResult::Return(v) => return StatementResult::Return(v),
                StatementResult::Continue => continue,
                StatementResult::Break => break,
                StatementResult::Failure(why) => return StatementResult::Failure(why),
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
                StatementResult::Failure(why) => return StatementResult::Failure(why),
            }
        }
        StatementResult::Ok(Value::Nil)
    }
    fn exec_var(&mut self, id: &String, initializer: &Option<Expr>) -> StatementResult {
        let mut value = Value::Nil;
        if let Some(expr) = initializer {
            let res = match self.evaluate(&expr) {
                Ok(v) => v,
                Err(why) => return StatementResult::Failure(why),
            };
            value = res;
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
        let base_strukt = BaseStruct::new(members.to_vec(), name.clone());
        let factory = StructFactory::new(Rc::new(base_strukt));

        let strukt = Value::Callable(Rc::new(Box::new(factory)));

        let mut current_env = self.current.as_ref().borrow_mut();
        current_env.set(name.clone(), strukt.clone());
        StatementResult::Ok(strukt)
    }

    fn exec_enumdef(
        &mut self,
        name: &String,
        values: &Vec<(String, Option<Expr>)>,
    ) -> StatementResult {
        let mut i = 0.0;
        let mut variants: HashMap<String, f64> = HashMap::new();
        for value in values {
            let assoc_value = if let Some(expr) = &value.1 {
                let res = match self.evaluate(&expr) {
                    Ok(v) => v,
                    Err(why) => return StatementResult::Failure(why),
                };
                match res {
                    Value::Num(e) => e,
                    _ => {
                        return StatementResult::Failure(format!(
                            "Enum variants can only be associated to numbers!"
                        ))
                    }
                }
            } else {
                let ii = i;
                i += 1.0;
                ii
            };
            variants.insert(value.0.clone(), assoc_value);
        }
        let enumt = Value::Enum(name.clone(), variants);
        let ret = enumt.clone();
        self.current.borrow_mut().set(name.clone(), enumt);
        StatementResult::Ok(ret)
    }

    fn exec_return(&mut self, e: &Expr) -> StatementResult {
        let vr = self.evaluate(e);
        match vr {
            Ok(v) => StatementResult::Return(v),
            Err(why) => StatementResult::Failure(why),
        }
    }

    fn arithmetic(
        &mut self,
        le: &Expr,
        op: super::tokens::TokenType,
        re: &Expr,
    ) -> Result<Value, String> {
        match op {
            Plus => {
                let (l, r) = get_values_no_bs!(self.evaluate(le), self.evaluate(re));
                if l.is_numeric() && r.is_numeric() {
                    Ok(Value::Num(l.as_numeric() + r.as_numeric()))
                } else if l.is_string() || r.is_string() {
                    Ok(Value::Str(format!("{}{}", l.stringfiy(), r.stringfiy())))
                } else {
                    Err(format!(
                        "Unsummable values! {}, {}",
                        l.stringfiy(),
                        r.stringfiy()
                    ))
                }
            }
            Minus => {
                operation!(self.evaluate(le), -, self.evaluate(re), Num)
                // let l = self.evaluate(le);
                // let r = self.evaluate(re);
                // Value::Num(l.as_numeric() - r.as_numeric())
            }
            Star => operation!(self.evaluate(le), *, self.evaluate(re), Num),
            Slash => {
                let (l, r) = get_values_no_bs!(self.evaluate(le), self.evaluate(re));
                let divisor = r.as_numeric();
                Ok(Value::Num(if divisor != 0.0 {
                    l.as_numeric() / r.as_numeric()
                } else {
                    0.0
                }))
            }
            Less => operation!(self.evaluate(le), <, self.evaluate(re), Bool),
            LessEquals => operation!(self.evaluate(le), <=, self.evaluate(re), Bool),
            More => operation!(self.evaluate(le), >, self.evaluate(re), Bool),
            MoreEquals => operation!(self.evaluate(le), >=, self.evaluate(re), Bool),
            EqualsEquals => {
                let (l, r) = get_values_no_bs!(self.evaluate(le), self.evaluate(re));
                Ok(Value::Bool(
                    l.as_numeric()
                        .approx_eq(r.as_numeric(), F64Margin::default()),
                ))
            }
            BangEquals => operation!(self.evaluate(le), !=, self.evaluate(re), Bool),

            LessLess => {
                let (l, r) = get_values_no_bs!(self.evaluate(le), self.evaluate(re));
                Ok(Value::Num(
                    ((l.as_numeric() as u64) << (r.as_numeric() as u64)) as f64,
                ))
            }
            MoreMore => {
                let (l, r) = get_values_no_bs!(self.evaluate(le), self.evaluate(re));
                Ok(Value::Num(
                    ((l.as_numeric() as u64) >> (r.as_numeric() as u64)) as f64,
                ))
            }
            Ampersand => {
                let (l, r) = get_values_no_bs!(self.evaluate(le), self.evaluate(re));
                Ok(Value::Num(
                    ((l.as_numeric() as u64) & (r.as_numeric() as u64)) as f64,
                ))
            }
            Pipe => {
                let (l, r) = get_values_no_bs!(self.evaluate(le), self.evaluate(re));
                Ok(Value::Num(
                    ((l.as_numeric() as u64) | (r.as_numeric() as u64)) as f64,
                ))
            }
            And => {
                let l = get_value!(self.evaluate(le));
                if !Evaluator::is_true(&l) {
                    Ok(Value::Bool(false))
                } else {
                    self.evaluate(re)
                }
            }
            Or => {
                let l = get_value!(self.evaluate(le));
                if Evaluator::is_true(&l) {
                    Ok(Value::Bool(true))
                } else {
                    self.evaluate(re)
                }
            }
            _ => unreachable!(),
        }
    }

    fn do_call(&mut self, fun: &Expr, args: &Vec<Expr>) -> Result<Value, String> {
        let callable_maybe = get_value!(self.evaluate(fun));

        match callable_maybe {
            Value::Callable(call) => {
                if call.arity() != args.len() as i8 && call.arity() != -1 {
                    return Err(format!(
                        "Arguments differ in size! Expected {}, got {}",
                        call.arity(),
                        args.len()
                    ));
                }

                let mut args_evaluated: Vec<Value> = Vec::new();
                for arg in args {
                    let evaluated = get_value!(self.evaluate(arg));
                    args_evaluated.push(evaluated);
                }

                Ok(call.call(self, args_evaluated))
            }
            _ => Ok(Value::Nil),
        }
    }

    fn get(&mut self, l: &Expr, e: &Expr) -> Result<Value, String> {
        let index = e;
        let base = get_value!(self.evaluate(l));
        match base {
            Value::Collection(map) => {
                let id = match index {
                    Expr::Id(s) => s,
                    _ => return Err(format!("Collections are only indexed by strings")),
                };
                Ok(match map.get(id) {
                    Some(val) => val.clone(),
                    None => Value::Nil,
                })
            }
            Value::List(lis) => {
                let index = match index {
                    Expr::Num(n) => *n as usize,
                    _ => return Err(format!("Lists are only indexed by numbers")),
                };
                if index >= lis.len() {
                    return Err(format!("Index out of bounds"));
                }
                Ok(match lis.get(index) {
                    Some(val) => val.clone(),
                    None => Value::Nil,
                })
            }
            Value::StructInstance(inst) => {
                let id = match index {
                    Expr::Id(s) => s,
                    _ => return Err(format!("Structs are only indexed by strings")),
                };
                return Ok(match inst.get(id) {
                    Some(value) => value.clone(),
                    None => Value::Nil,
                });
            }
            Value::Enum(_, fields) => {
                let id = match index {
                    Expr::Id(s) => s,
                    _ => return Err(format!("Enums are only indexed by strings")),
                };

                return Ok(match fields.get(id) {
                    Some(n) => Value::Num(*n),
                    None => Value::Nil,
                });
            }
            _ => {
                return Err(format!("Invalid get target! {}", base.stringfiy()));
            }
        }
    }
    fn assign(&mut self, l: &Expr, r: &Expr) -> Result<Value, String> {
        let value = get_value!(self.evaluate(r));
        match l {
            Expr::Id(name) => {
                let mut current_env = self.current.as_ref().borrow_mut();
                current_env.set(name.clone(), value.clone());
            }
            Get(expr, id) => {
                let base = get_value!(self.evaluate(expr));;
                match base {
                    Value::Collection(mut map) => {
                        let id = match id.as_ref() {
                            Expr::Id(s) => s,
                            _ => return Err(format!("Collections are only indexed by strings")),
                        };

                        map.insert(id.clone(), value.clone());
                    }
                    Value::List(mut lis) => {
                        let index = match id.as_ref() {
                            Expr::Num(n) => *n as usize,
                            _ => return Err(format!("Lists are only indexed by numbers")),
                        };
                        if index >= lis.len() {
                            return Err(format!("Index out of bounds"));
                        }
                        lis.push(value.clone());
                        let _ = lis.swap_remove(index);
                    }
                    Value::StructInstance(mut inst) => {
                        let id = match id.as_ref() {
                            Expr::Id(s) => s,
                            _ => return Err(format!("Structs are only indexed by strings")),
                        };
                        if let Err(_) = inst.set(id, value.clone()) {
                            return Err(format!("{} is not a member of this struct", id));
                        }
                    }
                    _ => {
                        return Err(format!("Invalid assign target!"));
                    }
                }
            }
            _ => {
                return Err(format!("Invalid assign target!"));
            }
        }
        Ok(value)
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

    fn make_dict(&mut self, elts: &Vec<(String, Expr)>) -> Result<Value, String> {
        let mut v: HashMap<String, Value> = HashMap::new();
        for el in elts {
            let evaluated = get_value!(self.evaluate(&el.1));
            v.insert(el.0.clone(), evaluated);
        }
        Ok(Value::Collection(v))
    }

    fn make_list(&mut self, elts: &Vec<Expr>) -> Result<Value, String> {
        let mut v: Vec<Value> = Vec::new();
        for el in elts {
            let evaluated = get_value!(self.evaluate(&el));
            v.push(evaluated);
        }
        Ok(Value::List(v))
    }
}

impl Evaluate<StatementResult, Result<Value, String>> for Evaluator {
    fn execute_statement(&mut self, s: &Stmt) -> StatementResult {
        match s {
            Stmt::ExprStmt(e) => {
                let v = self.evaluate(&e);
                return match v {
                    Ok(vs) => StatementResult::Ok(vs),
                    Err(why) => StatementResult::Failure(why),
                };
            }
            Stmt::If(branches, else_block) => self.exec_if(branches, else_block),
            Stmt::While(cond, block) => self.exec_while(cond, block),
            Stmt::Block(stmts) => self.exec_block(stmts),
            Stmt::Var(id, expr) => self.exec_var(id, expr),
            Stmt::FunDef(name, params, block) => self.exec_fundef(name, params, block),
            Stmt::StructDef(name, members) => self.exec_structdef(name, members),
            Stmt::EnumDef(name, values) => self.exec_enumdef(name, values),
            Stmt::Return(expr) => self.exec_return(expr),
            Stmt::Import(module) => self.exec_import(module),
            Stmt::Break => StatementResult::Break,
            Stmt::Continue => StatementResult::Continue,
        }
    }

    fn evaluate(&mut self, e: &Expr) -> Result<Value, String> {
        match e {
            Expr::Num(n) => Ok(Value::Num(*n)),
            Expr::Str(s) => Ok(Value::Str(s.clone())),
            Expr::Bool(b) => Ok(Value::Bool(*b)),

            Unary(op, e) => match op {
                Plus => self.evaluate(e),
                Minus | Not => match self.evaluate(e) {
                    Ok(mut v) => v.negate(),
                    Err(why) => Err(why),
                },
                _ => unreachable!(),
            },
            Binary(l, op, r) => self.arithmetic(l, *op, r),
            Grouping(e) => self.evaluate(e),
            Id(name) => Ok(self.get_value(name)),
            Call(exp, args) => self.do_call(exp, args),
            Get(l, r) => self.get(l, r),
            Assign(l, r) => self.assign(l, r),
            LambdaDef(params, stmts) => Ok(self.lambda(params, stmts.clone())),
            DictDef(elts) => self.make_dict(elts),
            ListDef(elts) => self.make_list(elts),
            Expr::Nil => Ok(Value::Nil),
        }
    }
}
