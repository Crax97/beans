use super::node::Expr;
use super::node::Stmt;
use std::rc::Rc;
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
    Enum(String, Vec<(String, f64)>),
    Struct(BaseStruct),
    StructInstance(StructInstance),
    Nil,
}

pub struct Symbol {
    v: Rc<Value>,
    childs: HashMap<String, Symbol>,
}
impl Symbol {
    pub fn new (v : Rc<Value>) -> Symbol {
        Symbol {
            v,
            childs : HashMap::new()
        }
    }
    pub fn get(&self, id: &String) -> Option<&Symbol> {
        self.childs.get(id)
    }

    pub fn set(&mut self, id: String, sym: Symbol) {
        self.childs.insert(id, sym);
    }
}

pub struct Closure {
    env: Rc<RefCell<Env>>,
    params: Vec<String>,
    fun: Rc<Vec<Stmt>>,
}
impl Closure {
    pub fn new(fun: Rc<Vec<Stmt>>, env: Rc<RefCell<Env>>, params: Vec<String>) -> Closure {
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
    name: String,
}

impl BaseStruct {
    pub fn new(fields: Vec<String>, name : String) -> BaseStruct {
        BaseStruct { fields, name }
    }
}
impl Call for BaseStruct {
    fn call(&self, exprs: Vec<Expr>) -> Value {
        // evaluate expressions
        Value::StructInstance(StructInstance::new(HashMap::new(), Rc::new(Box::new(Value::Nil))))
    }
    fn arity(&self) -> usize {
        self.fields.len()
    }
}

pub struct StructInstance {
    parent : Rc<Box<Value>>,
    fields: HashMap<String, Expr>,
}

impl StructInstance {
    fn new(fields: HashMap<String, Expr>, parent : Rc<Box<Value>>) -> StructInstance {
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

pub struct Env {
    symbols: HashMap<String, Symbol>,
    enclosing: Option<Box<Env>>,
}

impl Env {
    pub fn new() -> Env {
        Env {
            symbols: HashMap::new(),
            enclosing: None,
        }
    }

    pub fn new_enclosing(enclosing: Box<Env>) -> Env {
        let mut e = Env::new();
        e.enclosing = Some(enclosing);
        e
    }
    pub fn get(&self, s: &String) -> Option<&Symbol> {
        if self.symbols.contains_key(s) {
            return self.symbols.get(s);
        }
        match &self.enclosing {
            Some(env) => env.get(s),
            None => None,
        }
    }

    pub fn set(&mut self, s: String, sym: Symbol) {
        self.symbols.insert(s, sym);
    }
}
