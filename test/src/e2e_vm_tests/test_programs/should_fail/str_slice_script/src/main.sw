script;

// str in constants is not allowed
const A: str = "abc";

// str in configurable is not allowed
configurable {
    B: str = "abc",
}

// str in main args is not allowed
// main returning str is not allowed
fn main(s: str) -> str {
    let a: str = "abc";
    let _b: str[3] = __to_str_array(a);
    
    s
}