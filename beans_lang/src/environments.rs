use super::evaluator::Evaluator;
use super::evaluator::StatementResult;

use super::node::Stmt;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub enum Value {
    Num(f64),
    Str(String),
    Bool(bool),
    Callable(Rc<Box<dyn Call>>),
    Enum(String, HashMap<String, f64>),
    StructInstance(StructInstance),
    Collection(HashMap<String, Value>),
    List(Vec<Value>),
    Nil,
}

pub struct Env {
    symbols: HashMap<String, Value>,
    enclosing: Option<Rc<RefCell<Env>>>,
}

pub struct StructFactory {
    base: Rc<BaseStruct>,
}

pub struct BaseStruct {
    fields: Vec<String>,
    name: String,
}

pub struct StructInstance {
    parent: Rc<BaseStruct>,
    fields: HashMap<String, Value>,
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
    fn to_string(&self) -> String {
        format!("native function")
    }
}

pub trait Call {
    fn call(&self, eval: &mut Evaluator, args: Vec<Value>) -> Value;
    fn arity(&self) -> usize;
    fn to_string(&self) -> String;
}

impl Clone for Value {
    fn clone(&self) -> Self {
        match self {
            Value::Num(n) => Value::Num(*n),
            Value::Str(s) => Value::Str(s.clone()),
            Value::Bool(b) => Value::Bool(*b),
            Value::Callable(call) => Value::Callable(call.clone()),
            Value::Enum(name, fields) => Value::Enum(name.clone(), fields.clone()),
            Value::StructInstance(inst) => Value::StructInstance(inst.clone()),
            Value::Collection(map) => Value::Collection(map.clone()),
            Value::List(lis) => Value::List(lis.clone()),
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
            Value::Num(_) => true,
            Value::Bool(_) => true,
            _ => false,
        }
    }
    pub fn is_string(&self) -> bool {
        match self {
            Value::Str(_) => true,
            _ => false,
        }
    }

    pub fn negate(&mut self) -> Result<Self, String> {
        match self {
            Value::Num(n) => Ok(Value::Num(-*n)),
            Value::Bool(b) => Ok(Value::Bool(!*b)),
            _ => Err(format!("Cannot negate value {}", self.stringfiy())),
        }
    }

    pub fn stringfiy(&self) -> String {
        match self {
            Value::Num(n) => format!("Num: {}", *n),
            Value::Str(s) => format!("Str: {}", s.clone()),
            Value::Bool(b) => format!("Bool: {}", *b),
            Value::Callable(call) => format!("Callable {}", call.to_string()),
            Value::Enum(name, fields) => {
                let mut fields_str = String::from("{\n");
                for field in fields {
                    fields_str =
                        format!("{}{}", fields_str, format!("\t{}: {},\n", field.0, field.1));
                }
                fields_str = format!("{}}}\n", fields_str);
                format!("Enum {} {}", name, fields_str)
            },
            Value::StructInstance(inst) => {
                let mut fields = format!("{{\n");
                for field in inst.parent.get_fields() {
                    fields = format!("\t{}{}", fields, field);
                }
                format!("{} Instance {}}}", inst.parent.get_name(), fields)
            }
            Value::Nil => format!("Nil"),
            Value::Collection(map) => format!("Collection, {} elements", map.len()),
            Value::List(lis) => format!("List, {} elements", lis.len()),
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
    fn to_string(&self) -> String {
        format!("<function>")
    }
}

impl StructFactory {
    pub fn new(base: Rc<BaseStruct>) -> StructFactory {
        StructFactory { base }
    }

    pub fn get_name(&self) -> &String {
        self.base.get_name()
    }

    pub fn get_fields(&self) -> &Vec<String> {
        self.base.get_fields()
    }
}

impl Call for StructFactory {
    fn call(&self, _eval: &mut Evaluator, args: Vec<Value>) -> Value {
        let mut map: HashMap<String, Value> = HashMap::new();

        for i in 0..self.base.get_fields().len() {
            let name = self.base.get_fields().get(i).unwrap().clone();
            let arg = args.get(i).unwrap().clone();
            map.insert(name, arg);
        }

        Value::StructInstance(StructInstance::new(map, self.base.clone()))
    }
    fn arity(&self) -> usize {
        self.base.get_fields().len()
    }

    fn to_string(&self) -> String{
        let mut fields = format!("{{\n");
        for field in self.get_fields() {
            fields = format!("\t{}{}", fields, field);
        }
        format!("Struct factory: {} {}}}", self.get_name(), fields)
    }
}

impl Clone for StructFactory {
    fn clone(&self) -> Self {
        StructFactory {
            base: self.base.clone(),
        }
    }
}

impl BaseStruct {
    pub fn new(fields: Vec<String>, name: String) -> BaseStruct {
        BaseStruct { fields, name }
    }

    pub fn get_fields(&self) -> &Vec<String> {
        &self.fields
    }

    pub fn get_name(&self) -> &String {
        &self.name
    }
}

impl StructInstance {
    pub fn new(fields: HashMap<String, Value>, parent: Rc<BaseStruct>) -> StructInstance {
        StructInstance { parent, fields }
    }

    pub fn get(&self, id: &String) -> Option<&Value> {
        self.fields.get(id)
    }

    pub fn set(&mut self, id: &String, v: Value) -> Result<(), ()> {
        if self.fields.contains_key(id) {
            self.fields.insert(id.clone(), v);
            Ok(())
        } else {
            Err(())
        }
    }
}

impl Clone for StructInstance {
    fn clone(&self) -> Self {
        StructInstance::new(self.fields.clone(), self.parent.clone())
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
