use super::node::Expr;
use std::cell::RefCell;
use std::collections::HashMap;

pub trait Call {
    fn call(&self, exprs: Vec<Expr>) -> Value;
    fn arity(&self) -> usize;
}

pub enum Value {
    Num(f64),
    Str(String),
    Bool(bool),
    Callable(Box<dyn Call>),
    Struct(BaseStruct),
    StructInstance(StructInstance),
    Nil,
}

pub struct Symbol {
    v: Value,
    childs: HashMap<String, Symbol>,
}
impl Symbol {
    pub fn get(&self, id: &String) -> Option<&Symbol> {
        self.childs.get(id)
    }

    pub fn set(&mut self, id: String, sym: Symbol) {
        self.childs.insert(id, sym);
    }
}

pub struct Closure {
    env: Box<Env>,
    params: Vec<String>,
    fun: Expr,
}
impl Closure {
    fn new(fun: Expr, env: Box<Env>, params: Vec<String>) -> Closure {
        Closure { env, params, fun }
    }
}
impl Call for Closure {
    fn call(&self, exprs: Vec<Expr>) -> Value {
        // evaluate exprs and execute fun
        Value::Num(0.0)
    }
    fn arity(&self) -> usize {
        self.params.len()
    }
}

pub struct BaseStruct {
    fields: Vec<String>,
}

impl BaseStruct {
    fn new(fields: Vec<String>) -> BaseStruct {
        BaseStruct { fields }
    }
}
impl Call for BaseStruct {
    fn call(&self, exprs: Vec<Expr>) -> Value {
        // evaluate expressions
        Value::StructInstance(StructInstance::new(HashMap::new()))
    }
    fn arity(&self) -> usize {
        self.fields.len()
    }
}

struct StructInstance {
    fields: HashMap<String, Expr>,
}

impl StructInstance {
    fn new(fields: HashMap<String, Expr>) -> StructInstance {
        StructInstance { fields }
    }
}

pub struct Env {
    symbols: HashMap<String, Symbol>,
    enclosing: Option<Box<Env>>,
}

impl Env {
    fn new() -> Env {
        Env {
            symbols: HashMap::new(),
            enclosing: None,
        }
    }

    fn new_enclosing(enclosing: Box<Env>) -> Env {
        let mut e = Env::new();
        e.enclosing = Some(enclosing);
        e
    }
    fn get(&self, s: &String) -> Option<&Symbol> {
        if self.symbols.contains_key(s) {
            return self.symbols.get(s);
        }
        match &self.enclosing {
            Some(env) => env.get(s),
            None => None,
        }
    }

    fn set(&mut self, s: String, sym: Symbol) {
        self.symbols.insert(s, sym);
    }
}
