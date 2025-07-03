// This test only shows that `panic` properly compiles in predicates.
// Testing for actual panicking is done in SDK harness tests.
predicate;

#[error_type]
enum ErrorEnum {
    #[error(m = "Error A.")]
    A: (),
}

fn main() -> bool {
    panic ErrorEnum::A;
}