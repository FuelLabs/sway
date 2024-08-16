use std::fmt;

#[derive(Eq, PartialEq, Hash, Debug, Clone, Copy, PartialOrd, Ord)]
pub enum IntegerBits {
    Eight,
    Sixteen,
    ThirtyTwo,
    SixtyFour,
    V256,
}

impl IntegerBits {
    /// Returns if `v` would overflow using `self` bits or not.
    pub fn would_overflow(&self, v: u64) -> bool {
        if v == 0 {
            return false;
        }

        let needed_bits = v.ilog2() + 1;
        let bits = match self {
            IntegerBits::Eight => 8,
            IntegerBits::Sixteen => 16,
            IntegerBits::ThirtyTwo => 32,
            IntegerBits::SixtyFour => 64,
            IntegerBits::V256 => return false,
        };

        needed_bits > bits
    }
}

#[test]
fn would_overflow_tests() {
    assert!(!IntegerBits::Eight.would_overflow(0));

    assert!(!IntegerBits::Eight.would_overflow(0xFF));
    assert!(IntegerBits::Eight.would_overflow(0x100));

    assert!(!IntegerBits::Sixteen.would_overflow(0xFFFF));
    assert!(IntegerBits::Sixteen.would_overflow(0x10000));

    assert!(!IntegerBits::ThirtyTwo.would_overflow(0xFFFFFFFF));
    assert!(IntegerBits::ThirtyTwo.would_overflow(0x100000000));

    assert!(!IntegerBits::SixtyFour.would_overflow(0xFFFFFFFFFFFFFFFF));
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
