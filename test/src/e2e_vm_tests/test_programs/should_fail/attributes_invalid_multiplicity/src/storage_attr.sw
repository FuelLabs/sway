library;

pub struct S { }

impl S {
    #[storage(read, write)]
    pub fn ok() { }

    #[storage(read)]
    #[storage(write)]
    #[storage(read, write), storage(read)]
    #[storage(write)]
    pub fn not_ok() { }
}