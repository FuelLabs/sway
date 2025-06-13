script;

// This test covers all possible usages of the `panic` expression,
// and their handling in unit tests.
// It also includes tests for `revert` and error signaling through `assert`ions and `require`s.

#[error_type]
enum TestError {
    #[error(m = "Error A has occurred.")]
    A: (),
    #[error(m = "Error B has occurred, with a boolean value.")]
    B: bool,
    #[error(m = "")]
    C: str,
    #[error(m = "    ")]
    D: str,
}

fn main() { }

#[test]
fn passing_dbgs_and_logs() {
    let _ = __dbg("This is a passing test containing `__dbg` outputs.");
    let x = 42;
    let _ = __dbg(x);

    log("This is a log from the passing test.");
    log(x);
}

#[test]
fn passing_no_dbgs_or_logs() { }

#[test]
fn failing_revert_intrinsic() {
    __revert(112233);
}

#[test]
fn failing_revert_function_with_dbgs_and_logs() {
    let _ = __dbg("Reverting in a test function.");
    let revert_code = 332211;
    let _ = __dbg(revert_code);

    log("This is a log from the reverting test.");
    revert(revert_code);
}

#[test]
fn failing_error_signal_assert() {
    let _ = __dbg(TestError::A);
    assert(false);
}

#[test]
fn failing_error_signal_assert_eq() {
    let _ = __dbg("This is a `__dbg` before the failing assert_eq.");
    log("We will get logged the asserted values and this message.");
    assert_eq(1111, 2222);
}

#[test]
fn failing_error_signal_assert_ne() {
    let _ = __dbg("This is a `__dbg` before the failing assert_ne.");
    log("We will get logged the asserted values and this message.");
    assert_ne(3333, 3333);
}

#[test]
fn failing_error_signal_require_str_error() {
    require(false, "This is an error message in a `require` call.");
}

#[test]
fn failing_error_signal_require_enum_error() {
    require(false, TestError::B(true));
}

#[test]
fn failing_panic_no_arg() {
    panic;
}

#[test]
fn failing_panic_unit_arg() {
    panic ();
}

#[test]
fn failing_panic_const_eval_str_arg() {
    panic "Panicked with a string argument.";
}

#[test]
fn failing_panic_const_eval_empty_str_arg() {
    panic "";
}

#[test]
fn failing_panic_const_eval_whitespace_str_arg() {
    panic "    ";
}

#[test]
fn failing_panic_non_const_eval_str_arg() {
    panic non_const_eval_str("Panicked with a non-const evaluated string argument.");
}

#[test]
fn failing_panic_non_const_eval_str_empty_arg() {
    panic non_const_eval_str("");
}

#[test]
fn failing_panic_non_const_eval_str_whitespace_arg() {
    panic non_const_eval_str("    ");
}

#[test]
fn failing_panic_error_enum_arg() {
    panic TestError::B(true);
}

#[test]
fn failing_panic_error_enum_arg_with_empty_msg() {
    panic TestError::C("This is an error with an empty error message.");
}

#[test]
fn failing_panic_error_enum_arg_with_whitespace_msg() {
    panic TestError::D("This is an error with a whitespace error message.");
}

fn non_const_eval_str(error: str) {
    panic error;
}