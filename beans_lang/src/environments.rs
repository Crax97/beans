use super::evaluator::Evaluator;
use super::evaluator::StatementResult;

use super::node::Stmt;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_env_enclosing() {
        let mut env = Env::new();
        env.bind("x", Value::Num(42.0));
        let enclosing = Env::new_enclosing(Rc::new(RefCell::new(env)));
        assert!(enclosing.get(&"x".to_string()).as_numeric() == 42.0);
    }
}

pub enum Value {
    Num(f64),
    Str(String),
    Bool(bool),
    Callable(Rc<Box<dyn Call>>),
    Enum(String, HashMap<String, f64>),
    StructInstance(StructInstance),
    Collection(Rc<RefCell<HashMap<String, Value>>>),
    List(Vec<Value>),
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
            Value::StructInstance(inst) => Value::StructInstance(inst.clone()),
            Value::Collection(map) => Value::Collection(map.clone()),
            Value::List(lis) => Value::List(lis.clone()),
            Value::Nil => Value::Nil,
        }
    }
}

pub trait Call {
    fn call(&self, eval: &mut Evaluator, args: Vec<Value>) -> Value;
    fn arity(&self) -> i8;
    fn to_string(&self) -> String;
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
    arity: i8,
}

impl NativeFn {
    pub fn new(fun: fn(Vec<Value>) -> Value, arity: i8) -> NativeFn {
        NativeFn { fun, arity }
    }
}

impl Call for NativeFn {
    fn call(&self, _eval: &mut Evaluator, args: Vec<Value>) -> Value {
        (self.fun)(args)
    }
    fn arity(&self) -> i8 {
        self.arity
    }
    fn to_string(&self) -> String {
        format!("native function")
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
            _ => Err(format!("Cannot negate value {}", self.stringify())),
        }
    }

    pub fn stringify(&self) -> String {
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
            }
            Value::StructInstance(inst) => {
                let mut fields = format!("{{\n");
                for field in inst.parent.get_fields() {
                    fields = format!("\t{}{}", fields, field);
                }
                format!("{} Instance {}}}", inst.parent.get_name(), fields)
            }
            Value::Nil => format!("Nil"),
            Value::Collection(map) => {
                let borrowed_map = map.borrow();
                let mut content = String::new();
                for el in borrowed_map.iter() {
                    let key = el.0;
                    let value = el.1;
                    content = format!("{}\t{} : {}\n", content, key, value.stringify())
                }

                format!("Collection: {{\n{}\n}}", content)
            }
            Value::List(lis) => format!("List, {} elements", lis.len()),
        }
    }

    pub fn string_repr(&self) -> String {
        match self {
            Value::Num(n) => format!("{}", *n),
            Value::Str(s) => format!("{}", s.clone()),
            Value::Bool(b) => format!("{}", *b),
            _ => self.stringify(),
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
            let name = self.params.get(i as usize).unwrap();
            let arg = args.get(i as usize).unwrap();
            call_env.insert(name.clone(), arg.clone());
        }

        let result = eval.evaluate_in_env(&self.fun, call_env);
        match result {
            StatementResult::Return(e) => e,
            StatementResult::Ok(_) => Value::Nil,
            StatementResult::Failure(why) => panic!(format!("Failure! {}", why)),
            _ => panic!("Cannot break or continue inside function!"),
        }
    }
    fn arity(&self) -> i8 {
        self.params.len() as i8
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
    fn arity(&self) -> i8 {
        self.base.get_fields().len() as i8
    }

    fn to_string(&self) -> String {
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
        if self.symbols.contains_key(&s) {
            self.symbols.insert(s, v);
        } else if let Some(env) = &self.enclosing {
            let mut env_enclosing = env.as_ref().borrow_mut();
            env_enclosing.set(s, v);
        } else {
            println!("Undefined value! {}", s);
        }
    }

    pub fn insert(&mut self, s: String, v: Value) {
        self.symbols.insert(s, v);
    }

    pub fn bind(&mut self, name: &str, val: Value) -> &Self {
        self.insert(String::from(name), val);
        self
    }

    pub fn add_constant(&mut self, name: &str, val: Value) -> &Self {
        self.set(String::from(name), val);
        self
    }

    pub fn make_callable(fun: fn(Vec<Value>) -> Value, arity: i8) -> Value {
        Value::Callable(Rc::new(Box::new(NativeFn::new(fun, arity))))
    }

    pub fn build_stdlib(&mut self) {
        let print = Env::make_callable(
            |vals: Vec<Value>| {
                for val in vals {
                    print!("{} ", val.string_repr());
                }
                print!("\n");
                Value::Nil
            },
            -1,
        );

        // self.bind("pwd", Env::make_callable(
        //     |_| {
        //         let current_dir = std::env::current_dir().unwrap();
        //         let current_dir = current_dir.to_str().unwrap();
        //         Value::Str(current_dir.to_string())
        //     }, 0));

        self.bind("print", print);

        let mut math: HashMap<String, Value> = HashMap::new();
        math.insert(String::from("PI"), Value::Num(std::f64::consts::PI));
        math.insert(
            String::from("cos"),
            Env::make_callable(
                |vals| {
                    let n = vals.first().unwrap().as_numeric();
                    Value::Num(n.cos())
                },
                1,
            ),
        );
        math.insert(
            String::from("sin"),
            Env::make_callable(
                |vals| {
                    let n = vals.first().unwrap().as_numeric();
                    Value::Num(n.sin())
                },
                1,
            ),
        );
        math.insert(
            String::from("tan"),
            Env::make_callable(
                |vals| {
                    let n = vals.first().unwrap().as_numeric();
                    Value::Num(n.tan())
                },
                1,
            ),
        );
        math.insert(
            String::from("atan"),
            Env::make_callable(
                |vals| {
                    let n = vals.first().unwrap().as_numeric();
                    Value::Num(n.atan())
                },
                1,
            ),
        );
        math.insert(
            String::from("atan2"),
            Env::make_callable(
                |vals| {
                    let n = vals.first().unwrap().as_numeric();
                    let o = vals.get(1).unwrap().as_numeric();
                    Value::Num(n.atan2(o))
                },
                1,
            ),
        );
        math.insert(
            String::from("pow"),
            Env::make_callable(
                |vals| {
                    let n = vals.first().unwrap().as_numeric();
                    let o = vals.get(1).unwrap().as_numeric();
                    Value::Num(n.powf(o))
                },
                2,
            ),
        );
        math.insert(
            String::from("pow2"),
            Env::make_callable(
                |vals| {
                    let n = vals.first().unwrap().as_numeric();
                    Value::Num(n * n)
                },
                1,
            ),
        );
        math.insert(
            String::from("sqrt"),
            Env::make_callable(
                |vals| {
                    let n = vals.first().unwrap().as_numeric();
                    Value::Num(n.sqrt())
                },
                1,
            ),
        );
        math.insert(
            String::from("abs"),
            Env::make_callable(
                |vals| {
                    let n = vals.first().unwrap().as_numeric();
                    Value::Num(n.abs())
                },
                1,
            ),
        );

        self.bind("math", Value::Collection(Rc::new(RefCell::new(math))));
    }
}
