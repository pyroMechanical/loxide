#[test]
fn and_truth() {
    test_output!(
        "./test/logical_operator/and_truth.lox",
        "false\nnil\nok\nok\nok\n"
    );
}

#[test]
fn and() {
    test_output!(
        "./test/logical_operator/and.lox",
        "false\n1\nfalse\ntrue\n3\ntrue\nfalse\n"
    );
}

#[test]
fn or_truth() {
    test_output!(
        "./test/logical_operator/or_truth.lox",
        "ok\nok\ntrue\n0\ns\n"
    );
}

#[test]
fn or() {
    test_output!(
        "./test/logical_operator/or.lox",
        "1\n1\ntrue\nfalse\nfalse\nfalse\ntrue\n"
    );
}
