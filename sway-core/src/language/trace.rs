/// The trace of a function suggests to the compiler whether or not a function should be backtraced.
#[derive(Clone, Debug, Copy, PartialEq, Eq, Hash)]
pub enum Trace {
    Always,
    Never,
}
