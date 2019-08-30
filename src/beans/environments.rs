use super::evaluator::Evaluate;
use super::evaluator::Evaluator;
use super::evaluator::StatementResult;
use super::node::Expr;
use super::node::Stmt;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

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
}

pub struct Symbol {
    v: Rc<Value>,
    childs: Rc<RefCell<HashMap<String, Rc<Symbol>>>>,
}
impl Symbol {
    pub fn new(v: Value) -> Symbol {
        let rc = Rc::new(v);
        Symbol {
            v: rc,
            childs: Rc::new(RefCell::new(HashMap::new())),
        }
    }

    pub fn get_value(&self) -> Value {
        self.v.as_ref().clone()
    }

    pub fn set_value(&mut self, v: Value) {
        self.v = Rc::new(v);
    }

    pub fn get(&self, id: &String) -> Rc<Symbol> {
        let nil_def = Rc::new(Symbol::new(Value::Nil));
        self.childs.borrow().get(id).unwrap_or(&nil_def).clone()
    }

    pub fn set(&mut self, id: String, sym: Symbol) {
        let mut mut_childs = self.childs.borrow_mut();
        mut_childs.insert(id, Rc::new(sym));
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
    fn call(&self, eval: &mut Evaluator, args: Vec<Value>) -> Value {
        let enclosing_rc = self.env.clone();

        let mut call_env = Env::new_enclosing(enclosing_rc);
        for i in 0..self.arity() {
            let name = self.params.get(i).unwrap();
            let arg = args.get(i).unwrap();
            call_env.set(name.clone(), Symbol::new(arg.clone()));
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

pub struct BaseStruct {
    fields: Vec<String>,
    name: String,
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

pub struct StructInstance {
    parent: Rc<Box<Value>>,
    fields: HashMap<String, Expr>,
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

pub struct Env {
    symbols: HashMap<String, Rc<Symbol>>,
    enclosing: Option<Rc<RefCell<Env>>>,
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
    pub fn get(&self, s: &String) -> Rc<Symbol> {
        if self.symbols.contains_key(s) {
            return self.symbols.get(s).unwrap().clone();
        }
        match &self.enclosing {
            Some(env) => env.as_ref().borrow().get(s),
            None => Rc::new(Symbol::new(Value::Nil)),
        }
    }

    pub fn set(&mut self, s: String, sym: Symbol) {
        self.symbols.insert(s, Rc::new(sym));
    }
}
