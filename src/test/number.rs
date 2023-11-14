#[test]
fn decimal_point_at_eof() {
    test_error!(
        "./test/number/decimal_point_at_eof.lox",
        "[line 2] Error at end: Expect property name after '.'.\n"
    );
}

#[test]
fn leading_dot() {
    test_error!(
        "./test/number/leading_dot.lox",
        "[line 2] Error at '.': Expect expression.\n"
    );
}

#[test]
fn literals() {
    test_output!(
        "./test/number/literals.lox",
        "123\n987654\n0\n-0\n123.456\n-0.001\n"
    );
}

#[test]
fn nan_equality() {
    test_output!(
        "./test/number/nan_equality.lox",
        "false\ntrue\nfalse\ntrue\n"
    );
}

#[test]
fn trailing_dot() {
    test_error!(
        "./test/number/trailing_dot.lox",
        "[line 2] Error at ';': Expect property name after '.'.\n"
    );
}
