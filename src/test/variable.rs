#[test]
fn collide_with_parameter() {
    test_error!(
        "./test/variable/collide_with_parameter.lox",
        "[line 2] Error at 'a': Already a variable with this name in this scope.\n"
    );
}

#[test]
fn duplicate_local() {
    test_error!(
        "./test/variable/duplicate_local.lox",
        "[line 3] Error at 'a': Already a variable with this name in this scope.\n"
    );
}

#[test]
fn duplicate_parameter() {
    test_error!(
        "./test/variable/duplicate_parameter.lox",
        "[line 2] Error at 'arg': Already a variable with this name in this scope.\n"
    );
}

#[test]
fn early_bound() {
    test_output!("./test/variable/early_bound.lox", "outer\nouter\n");
}

#[test]
fn in_middle_of_block() {
    test_output!(
        "./test/variable/in_middle_of_block.lox",
        "a\na b\na c\na b d\n"
    );
}

#[test]
fn in_nested_block() {
    test_output!("./test/variable/in_nested_block.lox", "outer\n");
}

#[test]
fn local_from_method() {
    test_output!("./test/variable/local_from_method.lox", "variable\n");
}

#[test]
fn redeclare_global() {
    test_output!("./test/variable/redeclare_global.lox", "nil\n");
}

#[test]
fn redefine_global() {
    test_output!("./test/variable/redefine_global.lox", "2\n");
}

#[test]
fn scope_reuse_in_different_blocks() {
    test_output!(
        "./test/variable/scope_reuse_in_different_blocks.lox",
        "first\nsecond\n"
    );
}

#[test]
fn shadow_and_local() {
    test_output!("./test/variable/shadow_and_local.lox", "outer\ninner\n");
}

#[test]
fn shadow_global() {
    test_output!("./test/variable/shadow_global.lox", "shadow\nglobal\n");
}

#[test]
fn shadow_local() {
    test_output!("./test/variable/shadow_local.lox", "shadow\nlocal\n");
}

#[test]
fn undefined_global() {
    test_error!(
        "./test/variable/undefined_global.lox",
        "Undefined variable 'notDefined'.\n"
    );
}

#[test]
fn undefined_local() {
    test_error!(
        "./test/variable/undefined_local.lox",
        "Undefined variable 'notDefined'.\n"
    );
}

#[test]
fn uninitialized() {
    test_output!("./test/variable/uninitialized.lox", "nil\n");
}

#[test]
fn unreached_undefined() {
    test_output!("./test/variable/unreached_undefined.lox", "ok\n");
}

#[test]
fn use_false_as_var() {
    test_error!(
        "./test/variable/use_false_as_var.lox",
        "[line 2] Error at 'false': Expect variable name.\n"
    );
}

#[test]
fn use_global_in_initializer() {
    test_output!("./test/variable/use_global_in_initializer.lox", "value\n");
}

#[test]
fn use_local_in_initializer() {
    test_error!(
        "./test/variable/use_local_in_initializer.lox",
        "[line 3] Error at 'a': Can't read local variable in its own initializer.\n"
    );
}

#[test]
fn use_nil_as_var() {
    test_error!(
        "./test/variable/use_nil_as_var.lox",
        "[line 2] Error at 'nil': Expect variable name.\n"
    );
}

#[test]
fn use_this_as_var() {
    test_error!(
        "./test/variable/use_this_as_var.lox",
        "[line 2] Error at 'this': Expect variable name.\n"
    );
}
