use std::fmt;

#[derive(Eq, PartialEq, Hash, Debug, Clone, Copy, PartialOrd, Ord)]
pub enum IntegerBits {
    Eight,
    Sixteen,
    ThirtyTwo,
    SixtyFour,
    Usize,
}

impl fmt::Display for IntegerBits {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use IntegerBits::*;
        let s = match self {
            Eight => "eight",
            Sixteen => "sixteen",
            ThirtyTwo => "thirty two",
            SixtyFour => "sixty four",
            Usize => "usize",
        };
        write!(f, "{s}")
    }
}
