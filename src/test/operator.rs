#[test]
fn add_bool_nil() {
    test_error!(
        "./test/operator/add_bool_nil.lox",
        "Operands must be two numbers or two strings.\n"
    );
}

#[test]
fn add_bool_num() {
    test_error!(
        "./test/operator/add_bool_num.lox",
        "Operands must be two numbers or two strings.\n"
    );
}

#[test]
fn add_bool_string() {
    test_error!(
        "./test/operator/add_bool_string.lox",
        "Operands must be two numbers or two strings.\n"
    );
}

#[test]
fn add_nil_nil() {
    test_error!(
        "./test/operator/add_nil_nil.lox",
        "Operands must be two numbers or two strings.\n"
    );
}

#[test]
fn add_num_nil() {
    test_error!(
        "./test/operator/add_num_nil.lox",
        "Operands must be two numbers or two strings.\n"
    );
}

#[test]
fn add_string_nil() {
    test_error!(
        "./test/operator/add_string_nil.lox",
        "Operands must be two numbers or two strings.\n"
    );
}

#[test]
fn add() {
    test_output!("./test/operator/add.lox", "579\nstring\n");
}

#[test]
fn comparison() {
    test_output!("./test/operator/comparison.lox", "true\nfalse\nfalse\ntrue\ntrue\nfalse\nfalse\nfalse\ntrue\nfalse\ntrue\ntrue\nfalse\nfalse\nfalse\nfalse\ntrue\ntrue\ntrue\ntrue\n");
}

#[test]
fn divide_nonnum_num() {
    test_error!(
        "./test/operator/divide_nonnum_num.lox",
        "Operands must be numbers.\n"
    );
}

#[test]
fn divide_num_nonnum() {
    test_error!(
        "./test/operator/divide_num_nonnum.lox",
        "Operands must be numbers.\n"
    );
}

#[test]
fn divide() {
    test_output!("./test/operator/divide.lox", "4\n1\n");
}

#[test]
fn equals_class() {
    test_output!(
        "./test/operator/equals_class.lox",
        "true\nfalse\nfalse\ntrue\nfalse\nfalse\nfalse\nfalse\n"
    );
}

#[test]
fn equals_method() {
    test_output!("./test/operator/equals_method.lox", "true\nfalse\n");
}

#[test]
fn equals() {
    test_output!(
        "./test/operator/equals.lox",
        "true\ntrue\nfalse\ntrue\nfalse\ntrue\nfalse\nfalse\nfalse\nfalse\n"
    );
}

#[test]
fn greater_nonnum_num() {
    test_error!(
        "./test/operator/greater_nonnum_num.lox",
        "Operands must be numbers.\n"
    );
}

#[test]
fn greater_num_nonnum() {
    test_error!(
        "./test/operator/greater_num_nonnum.lox",
        "Operands must be numbers.\n"
    );
}

#[test]
fn greater_or_equal_nonnum_num() {
    test_error!(
        "./test/operator/greater_or_equal_nonnum_num.lox",
        "Operands must be numbers.\n"
    );
}

#[test]
fn greater_or_equal_num_nonnum() {
    test_error!(
        "./test/operator/greater_or_equal_num_nonnum.lox",
        "Operands must be numbers.\n"
    );
}

#[test]
fn less_nonnum_num() {
    test_error!(
        "./test/operator/less_nonnum_num.lox",
        "Operands must be numbers.\n"
    );
}

#[test]
fn less_num_nonnum() {
    test_error!(
        "./test/operator/less_num_nonnum.lox",
        "Operands must be numbers.\n"
    );
}

#[test]
fn less_or_equal_nonnum_num() {
    test_error!(
        "./test/operator/less_or_equal_nonnum_num.lox",
        "Operands must be numbers.\n"
    );
}

#[test]
fn less_or_equal_num_nonnum() {
    test_error!(
        "./test/operator/less_or_equal_num_nonnum.lox",
        "Operands must be numbers.\n"
    );
}

#[test]
fn multiply_nonnum_num() {
    test_error!(
        "./test/operator/multiply_nonnum_num.lox",
        "Operands must be numbers.\n"
    );
}

#[test]
fn multiply_num_nonnum() {
    test_error!(
        "./test/operator/multiply_num_nonnum.lox",
        "Operands must be numbers.\n"
    );
}

#[test]
fn multiply() {
    test_output!("./test/operator/multiply.lox", "15\n3.702\n");
}

#[test]
fn negate_nonnum() {
    test_error!(
        "./test/operator/negate_nonnum.lox",
        "Operand must be a number.\n"
    );
}

#[test]
fn negate() {
    test_output!("./test/operator/negate.lox", "-3\n3\n-3\n");
}

#[test]
fn not_class() {
    test_output!("./test/operator/not_class.lox", "false\nfalse\n");
}

#[test]
fn not_equals() {
    test_output!(
        "./test/operator/not_equals.lox",
        "false\nfalse\ntrue\nfalse\ntrue\nfalse\ntrue\ntrue\ntrue\ntrue\n"
    );
}

#[test]
fn not() {
    test_output!(
        "./test/operator/not.lox",
        "false\ntrue\ntrue\nfalse\nfalse\ntrue\nfalse\nfalse\n"
    );
}

#[test]
fn subtract_nonnum_num() {
    test_error!(
        "./test/operator/subtract_nonnum_num.lox",
        "Operands must be numbers.\n"
    );
}

#[test]
fn subtract_num_nonnum() {
    test_error!(
        "./test/operator/subtract_num_nonnum.lox",
        "Operands must be numbers.\n"
    );
}

#[test]
fn subtract() {
    test_output!("./test/operator/subtract.lox", "1\n0\n");
}
