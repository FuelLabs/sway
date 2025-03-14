/// The inline of a function suggests to the compiler whether or not a function should be inlined.
#[derive(Clone, Debug, Copy, PartialEq, Eq, Hash)]
pub enum Inline {
    Always,
    Never,
}
