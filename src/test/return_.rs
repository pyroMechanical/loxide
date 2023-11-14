#[test]
fn after_else() {
    test_output!("./test/return/after_else.lox", "ok\n");
}

#[test]
fn after_if() {
    test_output!("./test/return/after_if.lox", "ok\n");
}

#[test]
fn after_while() {
    test_output!("./test/return/after_while.lox", "ok\n");
}

#[test]
fn at_top_level() {
    test_error!(
        "./test/return/at_top_level.lox",
        "[line 1] Error at 'return': Can't return from top-level code.\n"
    );
}

#[test]
fn in_function() {
    test_output!("./test/return/in_function.lox", "ok\n");
}

#[test]
fn in_method() {
    test_output!("./test/return/in_method.lox", "ok\n");
}

#[test]
fn return_nil_if_no_value() {
    test_output!("./test/return/return_nil_if_no_value.lox", "nil\n");
}
