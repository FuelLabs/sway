use std::fmt;

#[derive(Eq, PartialEq, Hash, Debug, Clone, Copy)]
pub enum IntegerBits {
    Eight,
    Sixteen,
    ThirtyTwo,
    SixtyFour,
}

impl fmt::Display for IntegerBits {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use IntegerBits::*;
        let s = match self {
            Eight => "eight",
            Sixteen => "sixteen",
            ThirtyTwo => "thirty two",
            SixtyFour => "sixty four",
        };
        write!(f, "{}", s)
    }
}
