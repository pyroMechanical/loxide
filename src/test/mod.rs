#[cfg(test)]
#[macro_export]
macro_rules! test_output {
    ($path: literal, $output: literal) => {
        use crate::run_file;
        use crate::vm::VM;
        let mut out = vec![];
        let mut err = vec![];
        let mut vm = VM::new(&mut out, &mut err);
        run_file(&mut vm, $path.to_string());
        //println!("{}", std::str::from_utf8(out.as_slice()).unwrap());
        assert_eq!(std::str::from_utf8(out.as_slice()).unwrap(), $output);
        assert_eq!(std::str::from_utf8(err.as_slice()).unwrap(), "");
    };
}
#[macro_export]
macro_rules! test_error {
    ($path: literal, $output: literal) => {
        use crate::run_file;
        use crate::vm::VM;
        let mut out = vec![];
        let mut err = vec![];
        let mut vm = VM::new(&mut out, &mut err);
        run_file(&mut vm, $path.to_string());
        //println!("{}", std::str::from_utf8(err.as_slice()).unwrap());
        assert_eq!(std::str::from_utf8(err.as_slice()).unwrap(), $output);
        assert_eq!(std::str::from_utf8(out.as_slice()).unwrap(), "");
    };
}
#[macro_export]
macro_rules! test_output_and_error {
    ($path: literal, $output: literal, $error: literal) => {
        use crate::run_file;
        use crate::vm::VM;
        let mut out = vec![];
        let mut err = vec![];
        let mut vm = VM::new(&mut out, &mut err);
        run_file(&mut vm, $path.to_string());
        //println!("{}", std::str::from_utf8(out.as_slice()).unwrap());
        assert_eq!(std::str::from_utf8(out.as_slice()).unwrap(), $output);
        assert_eq!(std::str::from_utf8(err.as_slice()).unwrap(), $error);
    };
}

#[test]
fn empty_file() {
    test_output!("./test/empty_file.lox", "");
}

#[test]
fn precedence() {
    test_output!(
        "./test/precedence.lox",
        "14\n8\n4\n0\ntrue\ntrue\ntrue\ntrue\n0\n0\n0\n0\n4\n"
    );
}

#[test]
fn unexpected_character() {
    test_error!(
        "./test/unexpected_character.lox",
        "[line 3] Error: Unexpected character.\n"
    );
}

mod assignment;
mod block;
mod bool;
mod call;
mod class;
mod closure;
mod comments;
mod constructor;
mod field;
mod for_;
mod function;
mod if_;
mod inheritance;
mod limit;
mod logical_operator;
mod method;
mod nil;
mod number;
mod operator;
mod print;
mod regression;
mod return_;
mod string;
mod super_;
mod this;
mod variable;
mod while_;
