#[test]
fn derive_attribute_checks() {
    let cases = trybuild::TestCases::new();
    cases.pass("tests/ui_pass/drop_with_attribute.rs");
    cases.pass("tests/ui_pass/custom_niche.rs");
    cases.compile_fail("tests/ui_fail/drop_without_attribute.rs");
    cases.compile_fail("tests/ui_fail/attribute_without_drop.rs");
    cases.compile_fail("tests/ui_fail/custom_niche_unsupported.rs");
    cases.compile_fail("tests/ui_fail/custom_niche_duplicate.rs");
}
