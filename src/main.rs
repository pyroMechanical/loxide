mod chunk;
mod compiler;
mod gc;
mod test;
mod object;
mod scanner;
mod value;
mod vm;

use vm::*;

fn repl<StdOut, StdErr>(vm: &mut VM<StdOut, StdErr>)
where
    StdOut: std::io::Write,
    StdErr: std::io::Write,
{
    let input = std::io::stdin();
    'repl: loop {
        let mut line = String::new();
        match input.read_line(&mut line) {
            Ok(n) => {
                if n == 0 {
                    break 'repl;
                }
                match vm.interpret(line) {
                    Ok(()) => (),
                    Err(_) => break 'repl,
                }
            }
            Err(_) => break 'repl,
        }
    }
}

pub fn run_file<StdOut, StdErr>(vm: &mut VM<StdOut, StdErr>, file_path: String)
where
    StdOut: std::io::Write,
    StdErr: std::io::Write,
{
    let file = std::fs::read_to_string(file_path.as_str());
    match file {
        Ok(source) => match vm.interpret(source) {
            Ok(()) => (),
            Err(_) => (),
        },
        Err(e) => eprintln!("could not read file {}: {}", file_path, e),
    };
}

fn main() {
    let _ = START_TIME.with(|start_time| start_time.get().elapsed());
    let mut stdout = std::io::stdout();
    let mut stderr = std::io::stderr();
    let mut vm = VM::new(&mut stdout, &mut stderr);
    let mut args = std::env::args();
    if args.len() == 1 {
        repl(&mut vm);
    } else if args.len() == 2 {
        run_file(&mut vm, args.nth(1).unwrap());
    } else {
        eprintln!("Usage: loxide [path]");
    }
}
