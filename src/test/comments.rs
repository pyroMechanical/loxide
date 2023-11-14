#[test]
fn line_at_eof() {
    test_output!("./test/comments/line_at_eof.lox", "ok\n");
}

#[test]
fn only_line_comment_and_line() {
    test_output!("./test/comments/only_line_comment_and_line.lox", "");
}

#[test]
fn only_line_comment() {
    test_output!("./test/comments/only_line_comment.lox", "");
}

#[test]
fn unicode() {
    test_output!("./test/comments/unicode.lox", "ok\n");
}
