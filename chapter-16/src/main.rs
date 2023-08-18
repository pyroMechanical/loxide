use std::rc::Rc;

mod chunk;
mod compiler;
mod value;
mod vm;

use vm::*;

fn repl(vm: &mut VM) {
    let input = std::io::stdin();
    'repl: loop {
        let mut line = String::new();
        match input.read_line(&mut line) {
            Ok(n) => {
                if n == 0 {
                    break 'repl;
                }
                vm.interpret(line);
            }
            Err(error) => break 'repl,
        }
    }
}

fn run_file(vm: &mut VM, file_path: String) {
    let file = std::fs::read_to_string(file_path.as_str());
    match file {
        Ok(source) => {
            vm.interpret(source);
        },
        Err(e) => eprintln!("could not read file {}: {}",  file_path, e)
    };
}

fn main() {
    let mut vm = VM::new();
    let mut args = std::env::args();
    if args.len() == 1 {
        repl(&mut vm);
    } else if args.len() == 2 {
        run_file(&mut vm, args.nth(1).unwrap());
    } else {
        eprintln!("Usage: rlox [path]");
    }
}
