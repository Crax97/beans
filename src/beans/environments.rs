use super::evaluator::Evaluate;
use super::evaluator::Evaluator;
use super::evaluator::StatementResult;
use super::node::Expr;
use super::node::Stmt;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Display;
use std::rc::Rc;

pub struct Env {
    symbols: HashMap<String, Value>,
    enclosing: Option<Rc<RefCell<Env>>>,
}

pub struct BaseStruct {
    fields: Vec<String>,
    name: String,
}

pub struct StructInstance {
    parent: Rc<Box<Value>>,
    fields: HashMap<String, Expr>,
}

pub struct Closure {
    env: Rc<RefCell<Env>>,
    params: Vec<String>,
    fun: Rc<Vec<Stmt>>,
}

pub struct NativeFn {
    fun: fn(Vec<Value>) -> Value,
    arity: usize,
}

impl NativeFn {
    pub fn new(fun: fn(Vec<Value>) -> Value, arity: usize) -> NativeFn {
        NativeFn { fun, arity }
    }
}

impl Call for NativeFn {
    fn call(&self, _eval: &mut Evaluator, args: Vec<Value>) -> Value {
        (self.fun)(args)
    }
    fn arity(&self) -> usize {
        self.arity
    }
}

pub trait Call {
    fn call(&self, eval: &mut Evaluator, args: Vec<Value>) -> Value;
    fn arity(&self) -> usize;
}

pub enum Value {
    Num(f64),
    Str(String),
    Bool(bool),
    Callable(Rc<Box<dyn Call>>),
    Enum(String, Vec<(String, f64)>),
    Struct(Rc<BaseStruct>),
    StructInstance(Rc<RefCell<StructInstance>>),
    Collection(HashMap<String, Value>),
    Nil,
}

impl Clone for Value {
    fn clone(&self) -> Self {
        match self {
            Value::Num(n) => Value::Num(*n),
            Value::Str(s) => Value::Str(s.clone()),
            Value::Bool(b) => Value::Bool(*b),
            Value::Callable(call) => Value::Callable(call.clone()),
            Value::Enum(name, fields) => Value::Enum(name.clone(), fields.clone()),
            Value::Struct(base) => Value::Struct(base.clone()),
            Value::StructInstance(inst) => Value::StructInstance(inst.clone()),
            Value::Collection(map) => Value::Collection(map.clone()),
            Value::Nil => Value::Nil,
        }
    }
}

impl Value {
    pub fn as_numeric(&self) -> f64 {
        match self {
            Value::Num(n) => *n,
            Value::Bool(b) => {
                if *b {
                    1.0
                } else {
                    0.0
                }
            }
            _ => unreachable!(),
        }
    }

    pub fn is_numeric(&self) -> bool {
        match self {
            Value::Num(n) => true,
            Value::Bool(b) => true,
            _ => false,
        }
    }
    pub fn stringfiy(&self) -> String {
        match self {
            Value::Num(n) => format!("Num: {}", *n),
            Value::Str(s) => format!("Str: {}", s.clone()),
            Value::Bool(b) => format!("Bool: {}", *b),
            Value::Callable(call) => format!("Callable"),
            Value::Enum(name, fields) => format!("Enum {}", name),
            Value::Struct(base) => format!("Struct"),
            Value::StructInstance(inst) => format!("StructInstance"),
            Value::Nil => format!("Nil"),
            Value::Collection(map) => format!("Collection, {} elements", map.len()),
        }
    }
}

impl Closure {
    pub fn new(fun: Rc<Vec<Stmt>>, env: Rc<RefCell<Env>>, params: Vec<String>) -> Closure {
        Closure { env, params, fun }
    }
}
impl Call for Closure {
    fn call(&self, eval: &mut Evaluator, args: Vec<Value>) -> Value {
        let enclosing_rc = self.env.clone();

        let mut call_env = Env::new_enclosing(enclosing_rc);
        for i in 0..self.arity() {
            let name = self.params.get(i).unwrap();
            let arg = args.get(i).unwrap();
            call_env.set(name.clone(), arg.clone());
        }

        let result = eval.evaluate_in_env(&self.fun, call_env);
        match result {
            StatementResult::Return(e) => e,
            StatementResult::Ok(_) => Value::Nil,
            _ => panic!("Cannot break or continue inside function!"),
        }
    }
    fn arity(&self) -> usize {
        self.params.len()
    }
}

impl BaseStruct {
    pub fn new(fields: Vec<String>, name: String) -> BaseStruct {
        BaseStruct { fields, name }
    }
}
impl Call for BaseStruct {
    fn call(&self, eval: &mut Evaluator, args: Vec<Value>) -> Value {
        // evaluate expressions
        Value::StructInstance(Rc::new(RefCell::new(StructInstance::new(
            HashMap::new(),
            Rc::new(Box::new(Value::Nil)),
        ))))
    }
    fn arity(&self) -> usize {
        self.fields.len()
    }
}

impl StructInstance {
    fn new(fields: HashMap<String, Expr>, parent: Rc<Box<Value>>) -> StructInstance {
        StructInstance { parent, fields }
    }

    fn get(&self, id: &String) -> Option<&Expr> {
        self.fields.get(id)
    }

    fn set(&self, id: &String, e: Expr) {
        if self.fields.contains_key(id) {
            //insert e into self
        } else {
            panic!()
        }
    }
}

impl Env {
    pub fn new() -> Env {
        Env {
            symbols: HashMap::new(),
            enclosing: None,
        }
    }

    pub fn new_enclosing(enclosing: Rc<RefCell<Env>>) -> Env {
        let mut e = Env::new();
        e.enclosing = Some(enclosing);
        e
    }
    pub fn get(&self, s: &String) -> Value {
        if self.symbols.contains_key(s) {
            let k = self.symbols.get(s).unwrap();
            return k.clone();
        }
        let v = match &self.enclosing {
            Some(env) => {
                let enclosing_env = env.as_ref().borrow();
                enclosing_env.get(s).clone()
            }
            None => Value::Nil,
        };
        v
    }

    pub fn set(&mut self, s: String, v: Value) {
        self.symbols.insert(s, v);
    }

    pub fn bind_fun(&mut self, name: &str, fun: fn(Vec<Value>) -> Value, arity: usize) -> &Self {
        let fun = NativeFn::new(fun, arity);
        self.set(String::from(name), Value::Callable(Rc::new(Box::new(fun))));
        self
    }

    pub fn add_constant(&mut self, name: &str, val: Value) -> &Self {
        self.set(String::from(name), val);
        self
    }
}
