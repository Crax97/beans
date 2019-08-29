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

struct Symbol {
    v: Value,
    childs: HashMap<String, RefCell<Box<Symbol>>>,
}
impl Symbol {
    pub fn new (v : Value) -> Symbol {
        Symbol {
            v,
            childs : HashMap::new()
        }
    }
    pub fn get(&self, id: &String) -> Option<&RefCell<Box<Symbol>>> {
        self.childs.get(id)
    }

    pub fn set(&mut self, id: String, sym: Symbol) {
        self.childs.insert(id, RefCell::new(Box::new(sym)));
    }
}

struct Closure {
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

struct BaseStruct {
    fields: Vec<String>,
    name: String,
}

impl BaseStruct {
    fn new(fields: Vec<String>, name : String) -> BaseStruct {
        BaseStruct { fields, name }
    }
}
impl Call for BaseStruct {
    fn call(&self, exprs: Vec<Expr>) -> Value {
        // evaluate expressions
        Value::StructInstance(StructInstance::new(HashMap::new(), RefCell::new(Box::new(Symbol::new(Value::Nil)))))
    }
    fn arity(&self) -> usize {
        self.fields.len()
    }
}

struct StructInstance {
    parent : RefCell<Box<Symbol>>,
    fields: HashMap<String, Expr>,
}

impl StructInstance {
    fn new(fields: HashMap<String, Expr>, parent : RefCell<Box<Symbol>>) -> StructInstance {
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

struct Env {
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
