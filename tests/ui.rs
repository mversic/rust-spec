#[test]
fn derive_attribute_checks() {
    let cases = trybuild::TestCases::new();
    cases.compile_fail("tests/ui_fail/raw_pointer_field.rs");
    cases.pass("tests/ui_pass/opaque_raw_pointer.rs");
    cases.pass("tests/ui_pass/drop_with_attribute.rs");
    cases.pass("tests/ui_pass/custom_niche.rs");
    cases.compile_fail("tests/ui_fail/drop_without_attribute.rs");
    cases.compile_fail("tests/ui_fail/invalid_attributes.rs");
    cases.compile_fail("tests/ui_fail/custom_niche_duplicate.rs");
}
