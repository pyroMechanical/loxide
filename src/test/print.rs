#[test]
fn missing_argument() {
    test_error!(
        "./test/print/missing_argument.lox",
        "[line 2] Error at ';': Expect expression.\n"
    );
}
