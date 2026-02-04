mod ast;
mod env;
mod interpreter;
mod parser;
mod stdlib;
mod token;

use env::Environment;
use interpreter::Interpreter;
use parser::Parser;
use std::cell::RefCell;
use std::rc::Rc;
use token::Lexer;

use std::env as std_env;
use std::fs;
use std::process;

fn main() {
    let args: Vec<String> = std_env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <filename>", args[0]);
        process::exit(1);
    }

    let filename = &args[1];
    let input = match fs::read_to_string(filename) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading file {}: {}", filename, e);
            process::exit(1);
        }
    };

    let lexer = Lexer::new(&input);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program();

    if !parser.errors.is_empty() {
        println!("Parser errors:");
        for err in parser.errors {
            println!("\t{}", err);
        }
        process::exit(1);
    } else {
        let env = Rc::new(RefCell::new(Environment::new()));

        // Register stdlib
        crate::stdlib::register_stdlib(Rc::clone(&env));

        // Add constants
        env.borrow_mut()
            .set("null".to_string(), crate::env::Object::Null);
        env.borrow_mut()
            .set("true".to_string(), crate::env::Object::Boolean(true));
        env.borrow_mut()
            .set("false".to_string(), crate::env::Object::Boolean(false));

        let mut interpreter = Interpreter::new();

        let result = interpreter.eval_program(&program, env);
        // Only print result if it's not Null (stdlib functions return Null mostly)
        // Or keep printing it.
        // println!("Interpreter Result: {}", result.inspect());
        // User asked to not print source, maybe they don't want result printed if it's just script execution?
        // But let's keep it for now or check if it's non-null.
        if result != crate::env::Object::Null {
            println!("Interpreter Result: {}", result.inspect());
        }
    }
}
