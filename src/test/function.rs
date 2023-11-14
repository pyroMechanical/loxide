#[test]
fn body_must_be_block() {
    test_error!("./test/function/body_must_be_block.lox", "[line 3] Error at '123': Expect '{' before function body.\n[line 4] Error at end: Expect '}' after block.\n");
}

#[test]
fn empty_body() {
    test_output!("./test/function/empty_body.lox", "nil\n");
}

#[test]
fn extra_arguments() {
    test_error!(
        "./test/function/extra_arguments.lox",
        "Expected 2 arguments but got 4.\n"
    );
}

#[test]
fn local_mutual_recursion() {
    test_error!(
        "./test/function/local_mutual_recursion.lox",
        "Undefined variable 'isOdd'.\n"
    );
}

#[test]
fn local_recursion() {
    test_output!("./test/function/local_recursion.lox", "21\n");
}

#[test]
fn missing_arguments() {
    test_error!(
        "./test/function/missing_arguments.lox",
        "Expected 2 arguments but got 1.\n"
    );
}

#[test]
fn missing_comma_in_parameters() {
    test_error!(
        "./test/function/missing_comma_in_parameters.lox",
        "[line 3] Error at 'c': Expect ')' after parameters.\n[line 4] Error at end: Expect '}' after block.\n"
    );
}

#[test]
fn mutual_recursion() {
    test_output!("./test/function/mutual_recursion.lox", "true\ntrue\n");
}

#[test]
fn nested_call_with_arguments() {
    test_output!(
        "./test/function/nested_call_with_arguments.lox",
        "hello world\n"
    );
}

#[test]
fn parameters() {
    test_output!(
        "./test/function/parameters.lox",
        "0\n1\n3\n6\n10\n15\n21\n28\n36\n"
    );
}

#[test]
fn print() {
    test_output!("./test/function/print.lox", "<fn foo>\n<native fn>\n");
}

#[test]
fn recursion() {
    test_output!("./test/function/recursion.lox", "21\n");
}

#[test]
fn too_many_arguments() {
    test_error!(
        "./test/function/too_many_arguments.lox",
        "[line 260] Error at 'a': Can't have more than 255 arguments.\n"
    );
}

#[test]
fn too_many_parameters() {
    test_error!(
        "./test/function/too_many_parameters.lox",
        "[line 257] Error at 'a': Can't have more than 255 parameters.\n"
    );
}
