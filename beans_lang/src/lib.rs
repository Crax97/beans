#[macro_use]
extern crate lazy_static;
extern crate float_cmp;

pub mod environments;
pub mod evaluator;
pub mod lexer;
mod node;
pub mod parser;
mod reader;
pub mod tokens;

#[cfg(test)]
mod tests {
    use super::beans;
    use super::environments::*;
    use super::evaluator::*;
    #[test]
    fn bind_sin() {
        let sin_v = |vs: Vec<Value>| {
            let d = vs.first().unwrap().as_numeric();
            Value::Num(d.sin())
        };

        let glob = beans::create_global();
        glob.as_ref()
            .borrow_mut()
            .bind("sin", Env::make_callable(sin_v, 1));

        let expr = String::from("sin(0.0);");
        let mut evaluator = beans::create_evaluator(glob);
        match beans::do_string(expr, &mut evaluator) {
            StatementResult::Ok(v) => assert!(v.as_numeric() == 0.0),
            _ => panic!("Failed evaluating sin(0)"),
        }
    }

    #[test]
    fn two_envs() {
        let glob = beans::create_global();

        let first = beans::create_enclosing(glob.clone());
        let sec = beans::create_enclosing(glob.clone());

        let cos_fun = |vs: Vec<Value>| {
            let d = vs.first().unwrap().as_numeric();
            Value::Num(d.cos())
        };

        let sin_fun = |vs: Vec<Value>| {
            let d = vs.first().unwrap().as_numeric();
            Value::Num(d.sin())
        };

        first
            .borrow_mut()
            .bind("cos", Env::make_callable(cos_fun, 1));
        sec
            .borrow_mut()
            .bind("sin", Env::make_callable(sin_fun, 1));

        let sin_expr = String::from("sin(0.0);");
        let cos_expr = String::from("cos(0.0);");

        let mut evaluator1 = beans::create_evaluator(first);
        let mut evaluator2 = beans::create_evaluator(sec);

        match beans::do_string(sin_expr.clone(), &mut evaluator2) {
            StatementResult::Ok(v) => assert!(v.as_numeric() == 0.0),
            _ => panic!("Failed evaluating sin(0)"),
        }
        match beans::do_string(cos_expr.clone(), &mut evaluator1) {
            StatementResult::Ok(v) => assert!(v.as_numeric() == 1.0),
            _ => panic!("Failed evaluating cos(0)"),
        }

        match beans::do_string(sin_expr, &mut evaluator1) {
            StatementResult::Ok(v) => match v {
                Value::Nil => {}
                _ => unreachable!(),
            },
            _ => {}
        }
        match beans::do_string(cos_expr, &mut evaluator2) {
            StatementResult::Ok(v) => match v {
                Value::Nil => {}
                _ => unreachable!(),
            },
            _ => {}
        }
    }
}

pub mod beans {

    use super::environments::*;
    use super::evaluator::Evaluate;
    use super::evaluator::Evaluator;
    use super::evaluator::StatementResult;
    use super::lexer::Lexer;
    use super::parser::Parser;
    use std::io::Read;
    use std::cell::RefCell;
    use std::rc::Rc;
    use std::fs::File;

    pub fn create_global() -> Rc<RefCell<Env>> {
        Rc::new(RefCell::new(Env::new()))
    }

    pub fn create_enclosing(env: Rc<RefCell<Env>>) -> Rc<RefCell<Env>> {
        Rc::new(RefCell::new(Env::new_enclosing(env.clone())))
    }

    pub fn create_evaluator(env: Rc<RefCell<Env>>) -> Evaluator {
        Evaluator::new_with_global(env)
    }

    pub fn do_string(program: String, evaluator: &mut Evaluator) -> StatementResult {

        let lexer = Lexer::new(program);
        let mut parser = Parser::new(lexer);
        let stmts = parser.parse();
        let mut result: StatementResult = StatementResult::Continue;
        for stmt in stmts {
            result = evaluator.execute_statement(&stmt);
        }
        result
    }

    pub fn do_file(file_path: &String, evaluator: &mut Evaluator) -> StatementResult {
        let mut file = match File::open(file_path) {
            Ok(file) => file,
            Err(why) => return StatementResult::Failure(format!("Error opening file {}: {}", file_path, why))
        };

        let mut file_content = String::new();
        file.read_to_string(&mut file_content);
        do_string(file_content, evaluator)

    }
   

}
