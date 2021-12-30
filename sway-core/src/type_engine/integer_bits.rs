#[derive(Eq, PartialEq, Hash, Debug, Clone, Copy)]
pub enum IntegerBits {
    Eight,
    Sixteen,
    ThirtyTwo,
    SixtyFour,
}

impl IntegerBits {
    pub(crate) fn friendly_str(&self) -> &'static str {
        use IntegerBits::*;
        match self {
            Eight => "eight",
            Sixteen => "sixteen",
            ThirtyTwo => "thirty two",
            SixtyFour => "sixty four",
        }
    }
}
