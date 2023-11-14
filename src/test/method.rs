#[test]
fn arity() {
    test_output!(
        "./test/method/arity.lox",
        "no args\n1\n3\n6\n10\n15\n21\n28\n36\n"
    );
}

#[test]
fn empty_block() {
    test_output!("./test/method/empty_block.lox", "nil\n");
}

#[test]
fn extra_arguments() {
    test_error!(
        "./test/method/extra_arguments.lox",
        "Expected 2 arguments but got 4.\n"
    );
}

#[test]
fn missing_arguments() {
    test_error!(
        "./test/method/missing_arguments.lox",
        "Expected 2 arguments but got 1.\n"
    );
}

#[test]
fn not_found() {
    test_error!(
        "./test/method/not_found.lox",
        "Undefined property 'unknown'.\n"
    );
}

#[test]
fn print_bound_method() {
    test_output!("./test/method/print_bound_method.lox", "<fn method>\n");
}

#[test]
fn refer_to_name() {
    test_error!(
        "./test/method/refer_to_name.lox",
        "Undefined variable 'method'.\n"
    );
}

#[test]
fn too_many_arguments() {
    test_error!(
        "./test/method/too_many_arguments.lox",
        "[line 259] Error at 'a': Can't have more than 255 arguments.\n"
    );
}

#[test]
fn too_many_parameters() {
    test_error!(
        "./test/method/too_many_parameters.lox",
        "[line 258] Error at 'a': Can't have more than 255 parameters.\n"
    );
}
