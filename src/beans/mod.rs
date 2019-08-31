pub mod environments;
pub mod evaluator;
pub mod lexer;
mod node;
pub mod parser;
mod reader;
pub mod tokens;

#[cfg(test)]
mod tests {
    use super::super::beans::environments::*;
    use super::super::beans::evaluator::*;
    use super::beans;
    #[test]
    fn bind_sin() {
        let sin_v = |vs: Vec<Value>| {
            let d = vs.first().unwrap().as_numeric();
            Value::Num(d.sin())
        };

        let glob = beans::create_env();
        glob.as_ref().borrow_mut().bind_fun("sin", sin_v, 1);

        let expr = "sin(0.0);";
        let mut evaluator = beans::make_evaluator(glob);
        match beans::exec_string(expr, &mut evaluator) {
            StatementResult::Ok(v) => assert!(v.unwrap().as_numeric() == 0.0),
            _ => panic!("Failed evaluating sin(0)"),
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

    pub fn create_env() -> Rc<RefCell<Env>> {
        Rc::new(RefCell::new(Env::new()))
    }

    pub fn make_evaluator(global: Rc<RefCell<Env>>) -> Evaluator {
        Evaluator::new_with_global(global)
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
