#[test]
fn loop_too_large() {
    test_error!(
        "./test/limit/loop_too_large.lox",
        "[line 2351] Error at '}': Loop body too large.\n"
    );
}

#[test]
fn no_reuse_constants() {
    test_error!(
        "./test/limit/no_reuse_constants.lox",
        "[line 35] Error at '1': Too many constants in one chunk.\n"
    );
}

#[test]
fn stack_overflow() {
    test_error!("./test/limit/stack_overflow.lox", "Stack overflow.\n");
}

#[test]
fn too_many_constants() {
    test_error!(
        "./test/limit/too_many_constants.lox",
        "[line 35] Error at '\"oops\"': Too many constants in one chunk.\n"
    );
}

#[test]
fn too_many_locals() {
    test_error!(
        "./test/limit/too_many_locals.lox",
        "[line 52] Error at 'oops': Too many local variables in function.\n"
    );
}

#[test]
fn too_many_upvalues() {
    test_error!(
        "./test/limit/too_many_upvalues.lox",
        "[line 102] Error at 'oops': Too many closure variables in function.\n"
    );
}
