pub mod environments;
pub mod evaluator;
pub mod lexer;
mod node;
pub mod parser;
mod reader;
pub mod tokens;

#[cfg(test)]
mod tests {
    use super::super::*;
    use super::evaluator::Evaluate;
    use super::evaluator::StatementResult;
    use super::*;
    #[test]
    fn try_simple_expr() {
        let expr = "3 * 2;";
        let lexer = lexer::Lexer::new(String::from(expr));
        let mut parser = parser::Parser::new(lexer);
        let stmts = parser.parse();

        let mut evaluator = evaluator::Evaluator::new();
        for stmt in stmts {
            match evaluator.execute_statement(&stmt) {
                StatementResult::Ok(sv) => {
                    if let Some(v) = sv {
                        println!("{}", v.stringfiy());
                    }
                }
                _ => {}
            }
        }
    }

    fn exec_single_str(s: &str) -> StatementResult {
        let lexer = lexer::Lexer::new(String::from(s));
        let mut parser = parser::Parser::new(lexer);
        let stmts = parser.parse();

        let mut evaluator = evaluator::Evaluator::new();
        return evaluator.execute_statement(&stmts.first().unwrap());
    }

    fn exec_prog(s: &str) -> StatementResult {
        let lexer = lexer::Lexer::new(String::from(s));
        let mut parser = parser::Parser::new(lexer);
        let stmts = parser.parse();

        let mut evaluator = evaluator::Evaluator::new();
        let mut value: StatementResult = StatementResult::Break;
        for stmt in stmts {
            value = evaluator.execute_statement(&stmt);
        }
        value
    }

    #[test]
    fn try_logical() {
        match exec_single_str("2 == 2;") {
            StatementResult::Ok(v) => assert!(v.unwrap().as_numeric() == 1.0),
            _ => panic!("Wrong result in 2 == 2!"),
        }
        match exec_single_str("2 != 2;") {
            StatementResult::Ok(v) => assert!(v.unwrap().as_numeric() != 1.0),
            _ => panic!("Wrong result in 2 != 2!"),
        }
        match exec_single_str("2 < 3;") {
            StatementResult::Ok(v) => assert!(v.unwrap().as_numeric() == 1.0),
            _ => panic!("Wrong result in 2 < 3!"),
        }
        match exec_single_str("2 > 3;") {
            StatementResult::Ok(v) => assert!(v.unwrap().as_numeric() != 1.0),
            _ => panic!("Wrong result in 2 > 3!"),
        }
        match exec_single_str("(2 > 3) or (3 > 1);") {
            StatementResult::Ok(v) => assert!(v.unwrap().as_numeric() == 1.0),
            _ => panic!("Wrong result in 2 > 3!"),
        }
        match exec_single_str("(2 > 3) and (3 > 1);") {
            StatementResult::Ok(v) => assert!(v.unwrap().as_numeric() != 1.0),
            _ => panic!("Wrong result in 2 > 3!"),
        }
        match exec_single_str("(2 > 3) or (3 > 1);") {
            StatementResult::Ok(v) => assert!(v.unwrap().as_numeric() == 1.0),
            _ => panic!("Wrong result in 2 > 3!"),
        }
        match exec_single_str("((2 > 3) and (3 > 1)) or (2 + 3 - 5 * 6 < 0);") {
            StatementResult::Ok(v) => assert!(v.unwrap().as_numeric() == 1.0),
            _ => panic!("Wrong result in 2 > 3!"),
        }
        match exec_single_str("((2 > 3) and (3 > 1)) and (2 + 3 - 5 * 6 < 0);") {
            StatementResult::Ok(v) => assert!(v.unwrap().as_numeric() != 1.0),
            _ => panic!("Wrong result in 2 > 3!"),
        }
    }

    #[test]
    fn try_statements() {
        let if_example = "if 2 < 3 then 
                return 3.14;
            else
                return 6.28;
            end";
        match exec_single_str(if_example) {
            StatementResult::Return(val) => assert!(val.as_numeric() == 3.14),
            _ => panic!("Return failed"),
        }

        match exec_single_str(
            "if 2 < 3 then 
                    if true then
                        return 3.14;
                    end
            else
                return 6.28;
            end",
        ) {
            StatementResult::Return(val) => assert!(val.as_numeric() == 3.14),
            _ => panic!("Return failed"),
        }

        match exec_prog(
            "function return_1()
                return 1;
            end
            return_1();",
        ) {
            StatementResult::Ok(val) => assert!(val.unwrap().as_numeric() == 1.0),
            _ => panic!("Ok 1 failed"),
        }

        match exec_prog(
            "function return_from_if() 
                if true then
                    return 3.14;
                end
            end
            return_from_if();",
        ) {
            StatementResult::Ok(val) => assert!(val.unwrap().as_numeric() == 3.14),
            _ => panic!("Ok 2 failed"),
        }

        match exec_prog(
            "function recursive(n) 
                if n == 1 then
                    return 3.14;
                else
                    return recursive(1);
                end
            end
            recursive(10);",
        ) {
            StatementResult::Ok(val) => assert!(val.unwrap().as_numeric() == 3.14),
            _ => panic!("Ok 3 failed"),
        }
    }

    #[test]
    fn boolean() {
        let factorial_prog = "function boolean(n)
                if n == 3.14 or n == 6 then
                    return true;
                end
                return false;
            end
            boolean(3.14);";
        match exec_prog(factorial_prog) {
            StatementResult::Ok(v) => assert!(v.unwrap().as_numeric() == 1.0),
            _ => panic!("Failure on factorial(5)"),
        }
    }
    #[test]
    fn factorial() {
        let factorial_prog = "function factorial(n)
                if n == 1 or n == 0 then
                    return 1;
                end
                return n * factorial(n - 1);
            end
            factorial(5);";
        match exec_prog(factorial_prog) {
            StatementResult::Ok(v) => assert!(v.unwrap().as_numeric() == 120.0),
            _ => panic!("Failure on factorial(5)"),
        }
    }
}

pub mod beans {}
