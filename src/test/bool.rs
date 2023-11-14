#[test]
fn equality() {
    test_output!("./test/bool/equality.lox", "true\nfalse\nfalse\ntrue\nfalse\nfalse\nfalse\nfalse\nfalse\nfalse\ntrue\ntrue\nfalse\ntrue\ntrue\ntrue\ntrue\ntrue\n");
}

#[test]
fn not() {
    test_output!("./test/bool/not.lox", "false\ntrue\ntrue\n");
}
