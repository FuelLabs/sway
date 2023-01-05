script;

struct CustomType {
    name: str[3],
}

enum MyResult<T, E> {
    Ok: T,
    Err: E,
}

fn main() {
    sell_product();
}

fn sell_product() -> MyResult<bool, CustomType> {
    if false {
        return MyResult::Err(CustomType {
            name: "foo"
        });
    };

    return MyResult::Ok(false);
}
