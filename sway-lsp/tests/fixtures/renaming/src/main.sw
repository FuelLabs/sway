script;

mod test_mod;
use test_mod::{DeepEnum, DeepStruct};

enum Color {
    Red: (),
    Green: (),
    Blue: (),
}

struct Point {
    x: u32,
    y: u32,
}

fn add(x: u32, y: u32) -> u32 {
    x + y
}

fn main() {
    let c = Color::Red;
    let point = Point { x: 10, y: 20 };
    let n = add(point.x, point.y);
    let f = (c, point, n);

    // raw identifier syntax 
    let r#struct = ();
    let _ = r#struct;

    let _ = DeepStruct::new(30);
    let _ = DeepEnum::Number(40);
    let _ = test_mod::test_fun();

    // external modules can't be renamed
    let _ = std::constants::ZERO_B256;
    let _ = core::primitives::b256::min();
}
