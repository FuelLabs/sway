library;

struct EmptyStruct { }

struct Struct {
    x: u8,
}

fn implements_enum<T>(_t: T) where T: Enum { }
fn implements_enum_no_args<T>() where T: Enum { }

pub fn main() {
    implements_enum(0u64);
    implements_enum_no_args::<u64>();
    implements_enum(Struct { x: 0 });
    implements_enum_no_args::<Struct>();
    implements_enum(EmptyStruct { });
    implements_enum_no_args::<EmptyStruct>();
    implements_enum([0u8, 0, 0]);
    implements_enum_no_args::<[u8;3]>();
    implements_enum((0, 0, 0));
    implements_enum_no_args::<(u64,u64,u64)>();
    implements_enum(());
    implements_enum_no_args::<()>();
}
