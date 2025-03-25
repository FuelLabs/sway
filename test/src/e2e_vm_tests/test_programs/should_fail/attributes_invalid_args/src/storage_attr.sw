library;

pub struct S { }

impl S {
    #[storage(read, write)]
    pub fn ok() { }

    #[storage(red, writte)]
    #[storage(unknown_arg)]
    pub fn not_ok() { }
}