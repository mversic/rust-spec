use trybuild::TestCases;

#[test]
fn derive_attribute_checks() {
    let test_cases = TestCases::new();
    test_cases.pass("tests/ui_pass/*.rs");
    test_cases.compile_fail("tests/ui_fail/*.rs");
}
