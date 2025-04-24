library;

struct EmptyStruct { }

struct Struct {
    x: u8,
}

enum Enum {
    A: ()
}

fn implements_error<T>(_t: T) where T: Error { }
fn implements_error_no_args<T>() where T: Error { }

pub fn main() {
    implements_error(0u64);
    implements_error_no_args::<u64>();
    implements_error(Struct { x: 0 });
    implements_error_no_args::<Struct>();
    implements_error(EmptyStruct { });
    implements_error_no_args::<EmptyStruct>();
    implements_error([0u8, 0, 0]);
    implements_error_no_args::<[u8;3]>();
    implements_error((0, 0, 0));
    implements_error_no_args::<(u64,u64,u64)>();
    implements_error(Enum::A);
    implements_error_no_args::<Enum>();
}
