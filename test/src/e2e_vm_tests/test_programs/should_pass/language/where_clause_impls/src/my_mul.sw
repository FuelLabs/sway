library my_mul;

pub trait MyMul {
    fn my_mul(self, other: Self) -> Self;
}

impl MyMul for u8 {
    fn my_mul(self, other: Self) -> Self {
        self * other
    }
}

impl MyMul for u16 {
    fn my_mul(self, other: Self) -> Self {
        self * other
    }
}

impl MyMul for u32 {
    fn my_mul(self, other: Self) -> Self {
        self * other
    }
}

impl MyMul for u64 {
    fn my_mul(self, other: Self) -> Self {
        self * other
    }
}
