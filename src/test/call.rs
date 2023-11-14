#[test]
fn bool() {
    test_error!(
        "./test/call/bool.lox",
        "Can only call functions and classes.\n"
    );
}

#[test]
fn nil() {
    test_error!(
        "./test/call/nil.lox",
        "Can only call functions and classes.\n"
    );
}

#[test]
fn num() {
    test_error!(
        "./test/call/num.lox",
        "Can only call functions and classes.\n"
    );
}

#[test]
fn object() {
    test_error!(
        "./test/call/object.lox",
        "Can only call functions and classes.\n"
    );
}

#[test]
fn string() {
    test_error!(
        "./test/call/string.lox",
        "Can only call functions and classes.\n"
    );
}
