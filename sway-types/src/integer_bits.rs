use std::fmt;

#[derive(Eq, PartialEq, Hash, Debug, Clone, Copy, PartialOrd, Ord)]
pub enum IntegerBits {
    Eight,
    Sixteen,
    ThirtyTwo,
    SixtyFour,
    V256,
}

impl fmt::Display for IntegerBits {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use IntegerBits::{Eight, Sixteen, SixtyFour, ThirtyTwo, V256};
        let s = match self {
            Eight => "eight",
            Sixteen => "sixteen",
            ThirtyTwo => "thirty two",
            SixtyFour => "sixty four",
            V256 => "256",
        };
        write!(f, "{s}")
    }
}
