library;

pub struct S { }

impl S {
    #[storage(read = true, write = false)]
    pub fn not_ok() { }
}