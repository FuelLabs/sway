library;

#[fuzz(invalid_arg)]
fn invalid_fuzz_with_args() {
}

#[fuzz]
#[fuzz_param]
fn invalid_fuzz_param_no_args() {
}

#[fuzz]
#[fuzz_param(unknown_arg = "value")]
fn invalid_fuzz_param_unknown_arg() {
}