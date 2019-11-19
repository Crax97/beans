extern crate beans_lang;
#[macro_use]
extern crate structopt;

use beans_lang::environments::Env;
use beans_lang::evaluator::Evaluate;
use beans_lang::evaluator::StatementResult;
use beans_lang::*;
use std::cell::RefCell;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::rc::Rc;
use structopt::StructOpt;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dynamic_dict() {
        let prog = String::from(
            "
        var dic = {};
        dic.k = 42;
        dic.k;
        ",
        );

        let env = beans::create_global();
        let mut evaluator = beans::create_evaluator(env);
        match beans::do_string(prog, &mut evaluator) {
            StatementResult::Ok(v) => assert!(v.as_numeric() == 42.0),
            _ => panic!("Failure on dict!"),
        }
    }
}

#[derive(StructOpt)]
#[structopt(rename_all = "kebab-case")]
struct Args {
    /// If present, these files will be executed. Otherwise, the interpreter will be run in interactive mode
    pub files: Vec<String>,

    #[structopt(long = "no-stdlib")]
    pub no_stdlib: bool,
}
fn main() {
    let env = beans::create_global();
    let args = Args::from_args();

    if !args.no_stdlib {
        env.as_ref().borrow_mut().build_stdlib();
    }

    if args.files.len() > 0 {
        execute_files(env, &args.files)
    } else {
        run_interpreter(env);
    }
}

fn execute_files(global_env: Rc<RefCell<Env>>, file_names: &Vec<String>) {
    for file_name in file_names {
        match File::open(file_name) {
            Ok(mut file) => {
                let mut content: String = String::default();
                if let Err(_) = file.read_to_string(&mut content) {
                    println!("Error! Failure while reading {}'s contents!", file_name);
                    continue;
                }

                let mut lexer = lexer::Lexer::new(content);
                let mut parser = parser::Parser::new();

                while let Some(token) = lexer.next() {
                    parser.add_token(token.clone());
                }

                let file_env = beans::create_enclosing(global_env.clone());
                let mut evaluator = beans::create_evaluator(file_env);
                let stmts = parser.parse();

                if parser.error() {
                    println!("Skipping file due to a parse error");
                    continue;
                }
                for stmt in stmts {
                    match evaluator.execute_statement(&stmt) {
                        StatementResult::Ok(_) | StatementResult::Return(_) => {}
                        StatementResult::Failure(why) => {
                            println!("Failure: {}, skipping file.", why);
                            break;
                        }
                        _ => {
                            println!("Unexpected result! Skipping file");
                            break;
                        }
                    }
                }
            }
            Err(_) => println!("Error! Could not open file {}", file_name),
        }
    }
}

fn get_line() -> String {
    let mut line = String::new();
    loop {
        let mut buf: [u8; 1] = [0; 1];
        if let Err(_) = std::io::stdin().read_exact(&mut buf) {
            println!("Failure while reading from stdin");
            std::process::exit(1);
        }

        let ch = char::from(*buf.get(0).unwrap());
        line.push(ch);
        if ch == '\n' {
            break;
        }
    }
    line
}

fn run_interpreter(global_env: Rc<RefCell<Env>>) {
    let mut evaluator = evaluator::Evaluator::new_with_global(global_env.clone());

    loop {
        let mut parser = parser::Parser::new();
        while {
            if parser.get_scope_level() == 0 {
                print!("> ");
            } else {
                print!("{}... ", parser.get_scope_level());
            }
            std::io::stdout().flush().unwrap();
            let current_line = get_line();
            let mut lexer = lexer::Lexer::new(current_line);
            while let Some(token) = lexer.next() {
                parser.add_token(token.clone());
            }
            !parser.is_ready_to_parse()
        } {}

        let stmts = parser.parse();
        for stmt in stmts {
            match evaluator.execute_statement(&stmt) {
                StatementResult::Ok(v) => println!("{}", v.stringify()),
                StatementResult::Return(v) => println!("{}", v.stringify()),
                StatementResult::Failure(why) => println!("Failure: {}", why),
                _ => {
                    println!("Unexpected result while in interactive mode, stopping execution");
                    break;
                }
            }
        }
    }
}
