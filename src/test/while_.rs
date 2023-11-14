#[test]
fn class_in_body() {
    test_error!(
        "./test/while/class_in_body.lox",
        "[line 2] Error at 'class': Expect expression.\n"
    );
}

#[test]
fn closure_in_body() {
    test_output!("./test/while/closure_in_body.lox", "1\n2\n3\n");
}

#[test]
fn fun_in_body() {
    test_error!(
        "./test/while/fun_in_body.lox",
        "[line 2] Error at 'fun': Expect expression.\n"
    );
}

#[test]
fn return_closure() {
    test_output!("./test/while/return_closure.lox", "i\n");
}

#[test]
fn return_inside() {
    test_output!("./test/while/return_inside.lox", "i\n");
}

#[test]
fn syntax() {
    test_output!("./test/while/syntax.lox", "1\n2\n3\n0\n1\n2\n");
}

#[test]
fn var_in_body() {
    test_error!(
        "./test/while/var_in_body.lox",
        "[line 2] Error at 'var': Expect expression.\n"
    );
}
