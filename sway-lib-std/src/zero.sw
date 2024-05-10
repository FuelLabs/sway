library;

pub trait Zero {
    fn zero() -> Self;
    fn is_zero(self) -> bool;
}

impl Zero for u8 {
    fn zero() -> Self {
        0u8
    }

    fn is_zero(self) -> bool {
        self == 0u8
    }
}

impl Zero for u16 {
    fn zero() -> Self {
        0u16
    }

    fn is_zero(self) -> bool {
        self == 0u16
    }
}

impl Zero for u32 {
    fn zero() -> Self {
        0u32
    }

    fn is_zero(self) -> bool {
        self == 0u32
    }
}

impl Zero for u64 {
    fn zero() -> Self {
        0u64
    }

    fn is_zero(self) -> bool {
        self == 0u64
    }
}

impl Zero for u256 {
    fn zero() -> Self {
        0x00u256
    }

    fn is_zero(self) -> bool {
        self == 0x00u256
    }
}

impl Zero for b256 {
    fn zero() -> Self {
        0x0000000000000000000000000000000000000000000000000000000000000000
    }

    fn is_zero(self) -> bool {
        self == 0x0000000000000000000000000000000000000000000000000000000000000000
    }
}
