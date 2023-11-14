#[test]
fn arguments() {
    test_output!("./test/constructor/arguments.lox", "init\n1\n2\n");
}

#[test]
fn call_init_early_return() {
    test_output!(
        "./test/constructor/call_init_early_return.lox",
        "init\ninit\nFoo instance\n"
    );
}

#[test]
fn call_init_explicitly() {
    test_output!(
        "./test/constructor/call_init_explicitly.lox",
        "Foo.init(one)\nFoo.init(two)\nFoo instance\ninit\n"
    );
}

#[test]
fn default_arguments() {
    test_error!(
        "./test/constructor/default_arguments.lox",
        "Expected 0 arguments but got 3.\n"
    );
}

#[test]
fn default() {
    test_output!("./test/constructor/default.lox", "Foo instance\n");
}

#[test]
fn early_return() {
    test_output!(
        "./test/constructor/early_return.lox",
        "init\nFoo instance\n"
    );
}

#[test]
fn extra_arguments() {
    test_error!(
        "./test/constructor/extra_arguments.lox",
        "Expected 2 arguments but got 4.\n"
    );
}

#[test]
fn init_not_method() {
    test_output!(
        "./test/constructor/init_not_method.lox",
        "not initializer\n"
    );
}

#[test]
fn missing_arguments() {
    test_error!(
        "./test/constructor/missing_arguments.lox",
        "Expected 2 arguments but got 1.\n"
    );
}

#[test]
fn return_in_nested_function() {
    test_output!(
        "./test/constructor/return_in_nested_function.lox",
        "bar\nFoo instance\n"
    );
}

#[test]
fn return_value() {
    test_error!(
        "./test/constructor/return_value.lox",
        "[line 3] Error at 'return': Can't return a value from an initializer.\n"
    );
}
