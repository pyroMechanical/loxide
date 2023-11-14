#[test]
fn closure() {
    test_output!("./test/this/closure.lox", "Foo\n");
}

#[test]
fn nested_class() {
    test_output!(
        "./test/this/nested_class.lox",
        "Outer instance\nOuter instance\nInner instance\n"
    );
}

#[test]
fn nested_closure() {
    test_output!("./test/this/nested_closure.lox", "Foo\n");
}

#[test]
fn this_at_top_level() {
    test_error!(
        "./test/this/this_at_top_level.lox",
        "[line 1] Error at 'this': Can't use 'this' outside of a class.\n"
    );
}

#[test]
fn this_in_method() {
    test_output!("./test/this/this_in_method.lox", "baz\n");
}

#[test]
fn this_in_top_level_function() {
    test_error!(
        "./test/this/this_in_top_level_function.lox",
        "[line 2] Error at 'this': Can't use 'this' outside of a class.\n"
    );
}
