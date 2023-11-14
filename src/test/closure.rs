#[test]
fn assign_to_closure() {
    test_output!(
        "./test/closure/assign_to_closure.lox",
        "local\nafter f\nafter f\nafter g\n"
    );
}

#[test]
fn close_over_function_parameter() {
    test_output!(
        "./test/closure/close_over_function_parameter.lox",
        "param\n"
    );
}

#[test]
fn close_over_later_variable() {
    test_output!("./test/closure/close_over_later_variable.lox", "b\na\n");
}

#[test]
fn close_over_method_parameter() {
    test_output!("./test/closure/close_over_method_parameter.lox", "param\n");
}

#[test]
fn closed_closure_in_function() {
    test_output!("./test/closure/closed_closure_in_function.lox", "local\n");
}

#[test]
fn nested_closure() {
    test_output!("./test/closure/nested_closure.lox", "a\nb\nc\n");
}

#[test]
fn open_closure_in_function() {
    test_output!("./test/closure/open_closure_in_function.lox", "local\n");
}

#[test]
fn reference_closure_multiple_times() {
    test_output!(
        "./test/closure/reference_closure_multiple_times.lox",
        "a\na\n"
    );
}

#[test]
fn reuse_closure_slot() {
    test_output!("./test/closure/reuse_closure_slot.lox", "a\n");
}

#[test]
fn shadow_closure_with_local() {
    test_output!(
        "./test/closure/shadow_closure_with_local.lox",
        "closure\nshadow\nclosure\n"
    );
}

#[test]
fn unused_closure() {
    test_output!("./test/closure/unused_closure.lox", "ok\n");
}

#[test]
fn unused_later_closure() {
    test_output!("./test/closure/unused_later_closure.lox", "a\n");
}
