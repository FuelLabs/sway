/// The inline of a function suggests to the compiler whether or no a function should be inline.
#[derive(Clone, Debug, Copy, PartialEq, Eq, Hash)]
pub enum Inline {
    Always,
    Never,
}
