#[test]
fn constructor() {
    test_output!("./test/inheritance/constructor.lox", "value\n");
}

#[test]
fn inherit_from_function() {
    test_error!(
        "./test/inheritance/inherit_from_function.lox",
        "Superclass must be a class.\n"
    );
}

#[test]
fn inherit_from_nil() {
    test_error!(
        "./test/inheritance/inherit_from_nil.lox",
        "Superclass must be a class.\n"
    );
}

#[test]
fn inherit_from_number() {
    test_error!(
        "./test/inheritance/inherit_from_number.lox",
        "Superclass must be a class.\n"
    );
}

#[test]
fn inherit_methods() {
    test_output!("./test/inheritance/inherit_methods.lox", "foo\nbar\nbar\n");
}

#[test]
fn parenthesized_superclass() {
    test_error!(
        "./test/inheritance/parenthesized_superclass.lox",
        "[line 4] Error at '(': Expect superclass name.\n"
    );
}

#[test]
fn set_fields_from_base_class() {
    test_output!(
        "./test/inheritance/set_fields_from_base_class.lox",
        "foo 1\nfoo 2\nbar 1\nbar 2\nbar 1\nbar 2\n"
    );
}
