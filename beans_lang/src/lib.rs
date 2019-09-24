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
        glob.as_ref().borrow_mut().bind("sin", Env::make_callable(sin_v, 1));

        let expr = "sin(0.0);";
        let mut evaluator = beans::create_evaluator(glob);
        match beans::exec_string(expr, &mut evaluator) {
            StatementResult::Ok(v) => assert!(v.as_numeric() == 0.0),
            _ => panic!("Failed evaluating sin(0)"),
        }
    }

    #[test]
    fn two_envs() {
        let glob = beans::create_global();

        let first = beans::create_with_global(glob.clone());
        let sec = beans::create_with_global(glob.clone());

        let cos_fun = |vs: Vec<Value>| {
            let d = vs.first().unwrap().as_numeric();
            Value::Num(d.cos())
        };

        let sin_fun = |vs: Vec<Value>| {
            let d = vs.first().unwrap().as_numeric();
            Value::Num(d.sin())
        };

        first
            .as_ref()
            .borrow_mut()
            .bind("cos", Env::make_callable(cos_fun, 1));
        sec.as_ref()
            .borrow_mut()
            .bind("sin", Env::make_callable(sin_fun, 1));

        let sin_expr = "sin(0.0);";
        let cos_expr = "cos(0.0);";

        let mut evaluator1 = beans::create_evaluator(first);
        let mut evaluator2 = beans::create_evaluator(sec);

        match beans::exec_string(sin_expr, &mut evaluator2) {
            StatementResult::Ok(v) => assert!(v.as_numeric() == 0.0),
            _ => panic!("Failed evaluating sin(0)"),
        }
        match beans::exec_string(cos_expr, &mut evaluator1) {
            StatementResult::Ok(v) => assert!(v.as_numeric() == 1.0),
            _ => panic!("Failed evaluating cos(0)"),
        }

        match beans::exec_string(sin_expr, &mut evaluator1) {
            StatementResult::Ok(v) => match v {
                Value::Nil => {}
                _ => unreachable!(),
            },
            _ => {}
        }
        match beans::exec_string(cos_expr, &mut evaluator2) {
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
    use std::cell::RefCell;
    use std::rc::Rc;

    pub struct Bean {
        env: Rc<RefCell<Env>>,
    }

    pub fn create_global() -> Rc<RefCell<Env>> {
        Rc::new(RefCell::new(Env::new()))
    }

    pub fn create_with_global(global: Rc<RefCell<Env>>) -> Rc<RefCell<Env>> {
        Rc::new(RefCell::new(Env::new_enclosing(global)))
    }

    pub fn create_evaluator(env: Rc<RefCell<Env>>) -> Evaluator {
        Evaluator::new_with_global(env)
    }

    pub fn exec_string(program: &str, evaluator: &mut Evaluator) -> StatementResult {
        let lexer = Lexer::new(String::from(program));
        let mut parser = Parser::new(lexer);
        let stmts = parser.parse();
        let mut result: StatementResult = StatementResult::Continue;
        for stmt in stmts {
            result = evaluator.execute_statement(&stmt);
        }
        result
    }

}
