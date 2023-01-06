library my_add;

pub trait MyAdd {
    fn my_add(self, other: Self) -> Self;
}

impl MyAdd for u8 {
    fn my_add(self, other: Self) -> Self {
        self + other
    }
}

impl MyAdd for u16 {
    fn my_add(self, other: Self) -> Self {
        self + other
    }
}

impl MyAdd for u32 {
    fn my_add(self, other: Self) -> Self {
        self + other
    }
}

impl MyAdd for u64 {
    fn my_add(self, other: Self) -> Self {
        self + other
    }
}
