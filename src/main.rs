extern crate beans_lang;
#[macro_use]
extern crate structopt;

use beans_lang::environments::Env;
use beans_lang::evaluator::Evaluate;
use beans_lang::evaluator::StatementResult;
use beans_lang::*;
use std::cell::RefCell;
use std::env;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::rc::Rc;
use structopt::StructOpt;

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

    if (args.no_stdlib) {
        // No stdlib in global env
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

                let lexer = lexer::Lexer::new(content);
                let mut parser = parser::Parser::new(lexer);

                let file_env = beans::create_with_global(global_env.clone());
                let mut evaluator = beans::create_evaluator(file_env);

                for stmt in parser.parse() {
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
    let scope_in = ["function", "if", "while", "for"];
    let scope_out = ["end"];
    let mut current_scope = 0;

    loop {
        let mut program_complete = String::new();
        while {
            if current_scope == 0 {
                print!("> ");
            } else {
                print!("{}... ", current_scope);
            }
            std::io::stdout().flush().unwrap();
            let current_line = get_line();

            let should_scope_in = scope_in
                .iter()
                .map(|token| current_line.contains(token))
                .fold(false, |acc, n| acc || n);
            let should_scope_out = scope_out
                .iter()
                .map(|token| current_line.contains(token))
                .fold(false, |acc, n| acc || n);
            if should_scope_in {
                current_scope += 1;
            }
            if should_scope_out {
                if current_scope != 0 {
                    current_scope -= 1;
                }
            }

            program_complete.push_str(&current_line.as_ref());

            current_scope != 0
        } {}
        let mut parser = parser::Parser::new(lexer::Lexer::new(program_complete));

        for stmt in parser.parse() {
            match evaluator.execute_statement(&stmt) {
                StatementResult::Ok(v) => println!("{}", v.stringfiy()),
                StatementResult::Return(v) => println!("{}", v.stringfiy()),
                StatementResult::Failure(why) => println!("Failure: {}", why),
                _ => {
                    println!("Unexpected result while in interactive mode, stopping execution");
                    break;
                }
            }
        }
    }
}
