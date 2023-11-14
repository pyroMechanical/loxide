#[test]
fn bound_method() {
    test_output!("./test/super/bound_method.lox", "A.method(arg)\n");
}

#[test]
fn call_other_method() {
    test_output!(
        "./test/super/call_other_method.lox",
        "Derived.bar()\nBase.foo()\n"
    );
}

#[test]
fn call_same_method() {
    test_output!(
        "./test/super/call_same_method.lox",
        "Derived.foo()\nBase.foo()\n"
    );
}

#[test]
fn closure() {
    test_output!("./test/super/closure.lox", "Base\n");
}

#[test]
fn constructor() {
    test_output!(
        "./test/super/constructor.lox",
        "Derived.init()\nBase.init(a, b)\n"
    );
}

#[test]
fn extra_arguments() {
    test_output_and_error!(
        "./test/super/extra_arguments.lox",
        "Derived.foo()\n",
        "Expected 2 arguments but got 4.\n"
    );
}

#[test]
fn indirectly_inherited() {
    test_output!(
        "./test/super/indirectly_inherited.lox",
        "C.foo()\nA.foo()\n"
    );
}

#[test]
fn missing_arguments() {
    test_error!(
        "./test/super/missing_arguments.lox",
        "Expected 2 arguments but got 1.\n"
    );
}

#[test]
fn no_superclass_bind() {
    test_error!(
        "./test/super/no_superclass_bind.lox",
        "[line 3] Error at 'super': Can't use 'super' in a class with no superclass.\n"
    );
}

#[test]
fn no_superclass_call() {
    test_error!(
        "./test/super/no_superclass_call.lox",
        "[line 3] Error at 'super': Can't use 'super' in a class with no superclass.\n"
    );
}

#[test]
fn no_superclass_method() {
    test_error!(
        "./test/super/no_superclass_method.lox",
        "Undefined property 'doesNotExist'.\n"
    );
}

#[test]
fn parenthesized() {
    test_error!(
        "./test/super/parenthesized.lox",
        "[line 8] Error at ')': Expect '.' after 'super'.\n"
    );
}

#[test]
fn reassign_superclass() {
    test_output!(
        "./test/super/reassign_superclass.lox",
        "Base.method()\nBase.method()\n"
    );
}

#[test]
fn super_at_top_level() {
    test_error!("./test/super/super_at_top_level.lox", "[line 1] Error at 'super': Can't use 'super' outside of a class.\n[line 2] Error at 'super': Can't use 'super' outside of a class.\n");
}

#[test]
fn super_in_closure_in_inherited_method() {
    test_output!(
        "./test/super/super_in_closure_in_inherited_method.lox",
        "A\n"
    );
}

#[test]
fn super_in_inherited_method() {
    test_output!("./test/super/super_in_inherited_method.lox", "A\n");
}

#[test]
fn super_in_top_level_function() {
    test_error!(
        "./test/super/super_in_top_level_function.lox",
        "[line 1] Error at 'super': Can't use 'super' outside of a class.\n"
    );
}

#[test]
fn super_without_dot() {
    test_error!(
        "./test/super/super_without_dot.lox",
        "[line 6] Error at ';': Expect '.' after 'super'.\n"
    );
}

#[test]
fn this_in_superclass_method() {
    test_output!("./test/super/this_in_superclass_method.lox", "a\nb\n");
}
