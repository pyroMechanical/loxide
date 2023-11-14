#[test]
fn class_in_else() {
    test_error!(
        "./test/if/class_in_else.lox",
        "[line 2] Error at 'class': Expect expression.\n"
    );
}

#[test]
fn class_in_then() {
    test_error!(
        "./test/if/class_in_then.lox",
        "[line 2] Error at 'class': Expect expression.\n"
    );
}

#[test]
fn dangling_else() {
    test_output!("./test/if/dangling_else.lox", "good\n");
}

#[test]
fn else_() {
    test_output!("./test/if/else.lox", "good\ngood\nblock\n");
}

#[test]
fn fun_in_else() {
    test_error!(
        "./test/if/fun_in_else.lox",
        "[line 2] Error at 'fun': Expect expression.\n"
    );
}

#[test]
fn fun_in_then() {
    test_error!(
        "./test/if/fun_in_then.lox",
        "[line 2] Error at 'fun': Expect expression.\n"
    );
}

#[test]
fn if_() {
    test_output!("./test/if/if.lox", "good\nblock\ntrue\n");
}

#[test]
fn truth() {
    test_output!("./test/if/truth.lox", "false\nnil\ntrue\n0\nempty\n");
}

#[test]
fn var_in_else() {
    test_error!(
        "./test/if/var_in_else.lox",
        "[line 2] Error at 'var': Expect expression.\n"
    );
}

#[test]
fn var_in_then() {
    test_error!(
        "./test/if/var_in_then.lox",
        "[line 2] Error at 'var': Expect expression.\n"
    );
}
