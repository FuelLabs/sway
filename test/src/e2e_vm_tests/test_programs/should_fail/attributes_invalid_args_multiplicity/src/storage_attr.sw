library;

pub struct S { }

impl S {
    #[storage(read)]
    pub fn ok_1() { }

    #[storage(read, write)]
    pub fn ok_2() { }

    #[storage]
    #[storage()]
    #[storage(read, write, read)]
    pub fn not_ok() { }
}