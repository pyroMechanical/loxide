mod chunk;
mod compiler;
mod gc;
mod object;
mod scanner;
mod value;
mod vm;

use vm::*;
/*class Doughnut {
        cook() {
          print "Dunk in the fryer.";
          this.finish("sprinkles");
        }
      
        finish(ingredient) {
          print "Finish with " + ingredient;
        }
      }
      
      class Cruller < Doughnut {
        finish(ingredient) {
          // No sprinkles, always icing.
          super.finish("icing");
        }
      }
    var cruller = Cruller();
    cruller.cook(); */
#[test]
fn test_vm()
{
    let mut vm = VM::new();
    let _ = vm.interpret(
    r#"
    class Foo {
      getClosure() {
        fun closure() {
          return this.toString();
        }
        return closure;
      }
      toString() {
        return "Foo";
      }
    }
    var closure = Foo().getClosure();
    print closure();
    "#.to_string());
    println!("gc allocations: {}", crate::gc::allocations())
}

fn repl(vm: &mut VM) {
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

fn run_file(vm: &mut VM, file_path: String) {
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
    let _ = START_TIME.elapsed();
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