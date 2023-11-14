#[test]
fn empty() {
    test_output!("./test/block/empty.lox", "ok\n");
}

#[test]
fn scope() {
    test_output!("./test/block/scope.lox", "inner\nouter\n");
}
