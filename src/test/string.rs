#[test]
fn error_after_multiline() {
    test_error!(
        "./test/string/error_after_multiline.lox",
        "Undefined variable 'err'.\n"
    );
}

#[test]
fn literals() {
    test_output!("./test/string/literals.lox", "()\na string\nA~¶Þॐஃ\n");
}

#[test]
fn multiline() {
    test_output!("./test/string/multiline.lox", "1\n2\n3\n");
}

#[test]
fn unterminated() {
    test_error!(
        "./test/string/unterminated.lox",
        "[line 2] Error: Unterminated String.\n"
    );
}
