// trybuild walks the real filesystem (`glob` → `stat`); Miri’s default filesystem isolation
// reports that as an unsupported operation. Run `cargo test` (without Miri) for UI tests.
#![cfg(not(miri))]

#[test]
fn macro_ui() {
    let t = trybuild::TestCases::new();
    t.pass("tests/compile_pass/*.rs");
    t.compile_fail("tests/compile_fail/*.rs");
}
