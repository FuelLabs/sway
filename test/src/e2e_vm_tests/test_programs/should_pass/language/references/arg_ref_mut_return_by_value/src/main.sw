script;

struct S {
    value: u64
}

impl S {
    fn arg_by_ref_ret_by_value(ref mut self) -> Self {
        self.value = 1;
        self
    }
}

fn main() -> u64 {
    let mut s = S { value: 0 };
    s = s.arg_by_ref_ret_by_value();
    0
}
