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

fn sell_product() -> Result<bool, CustomType> {
    if false {
        return Result::Err(CustomType {
            name: "foo"
        });
    };

    return Result::Ok(false);
}
