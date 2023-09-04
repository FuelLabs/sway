script;

fn len_3(_s: str[3]) -> u64 {
    3
}

fn main() -> u64 {
    let a: str = "abc";
    let b: str[3] = __to_str_array("def");
    a.len() + len_3(b)
}
