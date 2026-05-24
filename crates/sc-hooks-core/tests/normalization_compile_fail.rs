#[test]
fn provider_hook_normalizer_rejects_external_impls() {
    let cases = trybuild::TestCases::new();
    cases.compile_fail("tests/ui/provider_hook_normalizer_external_impl.rs");
}
