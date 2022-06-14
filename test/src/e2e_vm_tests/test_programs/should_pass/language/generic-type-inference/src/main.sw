script;

struct CustomType {
    name: str[3],
}

enum Result<T, E> {
    Ok: T,
    Err: E,
}

fn main() {
  sell_product();
}

fn generic_1<T>(x: T) -> T {
    x
}

fn generic_2<U>(y: U) -> U {
    let x = generic_1(true);
    y
}

fn generic_3<V>(z: V) -> V {
    let x = generic_1(1u8);
    let y = generic_2(1u16);
    z
}

fn sell_product() -> Result<bool, CustomType> {
    if false {
        return Result::Err(CustomType {
            name: "foo"
        });
    };

    return Result::Ok(false);
}
