script;

struct Wrapper {
    name: str[9],
}

fn main() -> Wrapper {
    Wrapper {
        name: __to_str_array("fuel-labs"),
    }
}
