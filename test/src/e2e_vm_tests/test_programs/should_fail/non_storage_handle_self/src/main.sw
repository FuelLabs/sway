script;

#[allow(dead_code)]
struct S {}

impl S {
    // The only type ascription allowed for self is `core::experimental::storage::StorageHandle`
    fn foo(self: u64, foo: u64) {}
}

fn main() {}
