/// whether or not something is constantly evaluatable (if the result is known at compile
/// time)
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub(crate) enum IsConstant {
    Yes,
    No,
}
