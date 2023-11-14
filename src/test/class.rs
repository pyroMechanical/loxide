#[test]
fn empty() {
    test_output!("./test/class/empty.lox", "Foo\n");
}

#[test]
fn inherit_self() {
    test_error!(
        "./test/class/inherit_self.lox",
        "[line 1] Error at 'Foo': A class can't inherit from itself.\n"
    );
}

#[test]
fn inherited_method() {
    test_output!(
        "./test/class/inherited_method.lox",
        "in foo\nin bar\nin baz\n"
    );
}

#[test]
fn local_inherit_other() {
    test_output!("./test/class/local_inherit_other.lox", "B\n");
}

#[test]
fn local_inherit_self() {
    test_error!(
        "./test/class/local_inherit_self.lox",
        "[line 2] Error at 'Foo': A class can't inherit from itself.\n[line 5] Error at end: Expect '}' after block.\n"
    );
}

#[test]
fn local_reference_self() {
    test_output!("./test/class/local_reference_self.lox", "Foo\n");
}

#[test]
fn reference_self() {
    test_output!("./test/class/reference_self.lox", "Foo\n");
}
