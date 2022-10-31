script;

use std::assert::assert;

trait MyAdd {
	fn my_add(self, other: Self) -> Self;
}

trait MyMul {
	fn my_mul(self, other: Self) -> Self;
}

trait MyMath: MyAdd + MyMul {

} {
	fn my_double(self) -> Self {
		self.my_add(self)
	}

	fn my_exp(self) -> Self {
		self.my_mul(self)
	}
}

struct Data {
    value: u64,
}

impl MyAdd for Data {
    fn my_add(self, other: Self) -> Self {
        Data {
            value: self.value + other.value
        }
    }
}

impl MyMul for Data {
    fn my_mul(self, other: Self) -> Self {
        Data {
            value: self.value * other.value
        }
    }
}

impl MyMath for Data {}

fn main() -> bool {
    let a = Data {
        value: 3u64
    };
    let b = a.my_exp();
    let c = b.my_double();
    assert(c.value == 18);

    true
}
