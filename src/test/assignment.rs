#[test]
fn associativity() {
    test_output!("./test/assignment/associativity.lox", "c\nc\nc\n");
}

#[test]
fn global() {
    test_output!("./test/assignment/global.lox", "before\nafter\narg\narg\n");
}

#[test]
fn grouping() {
    test_error!(
        "./test/assignment/grouping.lox",
        "[line 2] Error at '=': Invalid assignment target.\n"
    );
}

#[test]
fn infix_operator() {
    test_error!(
        "./test/assignment/infix_operator.lox",
        "[line 3] Error at '=': Invalid assignment target.\n"
    );
}

#[test]
fn local() {
    test_output!("./test/assignment/local.lox", "before\nafter\narg\narg\n");
}

#[test]
fn prefix_operator() {
    test_error!(
        "./test/assignment/prefix_operator.lox",
        "[line 2] Error at '=': Invalid assignment target.\n"
    );
}

#[test]
fn syntax() {
    test_output!("./test/assignment/syntax.lox", "var\nvar\n");
}

#[test]
fn to_this() {
    test_error!(
        "./test/assignment/to_this.lox",
        "[line 3] Error at '=': Invalid assignment target.\n"
    );
}

#[test]
fn undefined() {
    test_error!(
        "./test/assignment/undefined.lox",
        "Undefined variable 'unknown'\n"
    );
}
