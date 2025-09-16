library;

struct A {
    very_long_field_name: u64,
}

struct B {
    very_long_field_name: A,
}

impl B {
    fn very_long_method_name(self, x: u64) -> B {
        B { very_long_field_name: A { very_long_field_name: x } }
    }
}

pub fn test() {
    42;
    poke(42);
    B { very_long_field_name: A { very_long_field_name: 0 } }.very_long_method_name(10);
    B {
        very_long_field_name: A {
            very_long_field_name: 0
        }
    }.very_long_method_name(10);
}

fn poke(x: u64) -> u64 {
    x
}