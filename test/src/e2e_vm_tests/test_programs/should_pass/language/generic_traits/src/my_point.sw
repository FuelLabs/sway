library my_point;

use ::my_double::MyDouble;

pub struct MyPoint<T> {
    x: T,
    y: T,
}

impl MyDouble<u64> for MyPoint<u64> {
    fn my_double(self, value: u64) -> u64 {
        (self.x*2) + (self.y*2) + (value*2)
    }
}
