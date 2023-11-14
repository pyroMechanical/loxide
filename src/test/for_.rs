#[test]
fn class_in_body() {
    test_error!(
        "./test/for/class_in_body.lox",
        "[line 2] Error at 'class': Expect expression.\n"
    );
}

#[test]
fn closure_in_body() {
    test_output!("./test/for/closure_in_body.lox", "4\n1\n4\n2\n4\n3\n");
}

#[test]
fn fun_in_body() {
    test_error!(
        "./test/for/fun_in_body.lox",
        "[line 2] Error at 'fun': Expect expression.\n"
    );
}

#[test]
fn return_closure() {
    test_output!("./test/for/return_closure.lox", "i\n");
}

#[test]
fn return_inside() {
    test_output!("./test/for/return_inside.lox", "i\n");
}

#[test]
fn scope() {
    test_output!("./test/for/scope.lox", "0\n-1\nafter\n0\n");
}

#[test]
fn statement_condition() {
    test_error!("./test/for/statement_condition.lox", "[line 3] Error at '{': Expect expression.\n[line 3] Error at ')': Expect ';' after expression.\n");
}

#[test]
fn statement_increment() {
    test_error!(
        "./test/for/statement_increment.lox",
        "[line 2] Error at '{': Expect expression.\n"
    );
}

#[test]
fn statement_initializer() {
    test_error!("./test/for/statement_initializer.lox", "[line 3] Error at '{': Expect expression.\n[line 3] Error at ')': Expect ';' after expression.\n");
}

#[test]
fn syntax() {
    test_output!(
        "./test/for/syntax.lox",
        "1\n2\n3\n0\n1\n2\ndone\n0\n1\n0\n1\n2\n0\n1\n"
    );
}

#[test]
fn var_in_body() {
    test_error!(
        "./test/for/var_in_body.lox",
        "[line 2] Error at 'var': Expect expression.\n"
    );
}
