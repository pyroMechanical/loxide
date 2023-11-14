#[test]
fn call_function_field() {
    test_output!("./test/field/call_function_field.lox", "bar\n1\n2\n");
}

#[test]
fn call_nonfunction_field() {
    test_error!(
        "./test/field/call_nonfunction_field.lox",
        "Can only call functions and classes.\n"
    );
}

#[test]
fn get_and_set_method() {
    test_output!(
        "./test/field/get_and_set_method.lox",
        "other\n1\nmethod\n2\n"
    );
}

#[test]
fn get_on_bool() {
    test_error!(
        "./test/field/get_on_bool.lox",
        "Only instances have properties.\n"
    );
}

#[test]
fn get_on_class() {
    test_error!(
        "./test/field/get_on_class.lox",
        "Only instances have properties.\n"
    );
}

#[test]
fn get_on_function() {
    test_error!(
        "./test/field/get_on_function.lox",
        "Only instances have properties.\n"
    );
}

#[test]
fn get_on_nil() {
    test_error!(
        "./test/field/get_on_nil.lox",
        "Only instances have properties.\n"
    );
}

#[test]
fn get_on_num() {
    test_error!(
        "./test/field/get_on_num.lox",
        "Only instances have properties.\n"
    );
}

#[test]
fn get_on_string() {
    test_error!(
        "./test/field/get_on_string.lox",
        "Only instances have properties.\n"
    );
}

#[test]
fn many() {
    test_output!(
            "./test/field/many.lox",
            "apple\napricot\navocado\nbanana\nbilberry\nblackberry\nblackcurrant\nblueberry\nboysenberry\ncantaloupe\ncherimoya\ncherry\nclementine\ncloudberry\ncoconut\ncranberry\ncurrant\ndamson\ndate\ndragonfruit\ndurian\nelderberry\nfeijoa\nfig\ngooseberry\ngrape\ngrapefruit\nguava\nhoneydew\nhuckleberry\njabuticaba\njackfruit\njambul\njujube\njuniper\nkiwifruit\nkumquat\nlemon\nlime\nlongan\nloquat\nlychee\nmandarine\nmango\nmarionberry\nmelon\nmiracle\nmulberry\nnance\nnectarine\nolive\norange\npapaya\npassionfruit\npeach\npear\npersimmon\nphysalis\npineapple\nplantain\nplum\nplumcot\npomegranate\npomelo\nquince\nraisin\nrambutan\nraspberry\nredcurrant\nsalak\nsalmonberry\nsatsuma\nstrawberry\ntamarillo\ntamarind\ntangerine\ntomato\nwatermelon\nyuzu\n"
        );
}

#[test]
fn method_binds_this() {
    test_output!("./test/field/method_binds_this.lox", "foo1\n1\n");
}

#[test]
fn method() {
    test_output!("./test/field/method.lox", "got method\narg\n");
}

#[test]
fn on_instance() {
    test_output!(
        "./test/field/on_instance.lox",
        "bar value\nbaz value\nbar value\nbaz value\n"
    );
}

#[test]
fn set_evaluation_order() {
    test_error!(
        "./test/field/set_evaluation_order.lox",
        "Undefined variable 'undefined1'.\n"
    );
}

#[test]
fn set_on_bool() {
    test_error!(
        "./test/field/set_on_bool.lox",
        "Only instances have fields.\n"
    );
}

#[test]
fn set_on_class() {
    test_error!(
        "./test/field/set_on_class.lox",
        "Only instances have fields.\n"
    );
}

#[test]
fn set_on_function() {
    test_error!(
        "./test/field/set_on_function.lox",
        "Only instances have fields.\n"
    );
}

#[test]
fn set_on_nil() {
    test_error!(
        "./test/field/set_on_nil.lox",
        "Only instances have fields.\n"
    );
}

#[test]
fn set_on_num() {
    test_error!(
        "./test/field/set_on_num.lox",
        "Only instances have fields.\n"
    );
}

#[test]
fn set_on_string() {
    test_error!(
        "./test/field/set_on_string.lox",
        "Only instances have fields.\n"
    );
}

#[test]
fn undefined() {
    test_error!("./test/field/undefined.lox", "Undefined property 'bar'.\n");
}
