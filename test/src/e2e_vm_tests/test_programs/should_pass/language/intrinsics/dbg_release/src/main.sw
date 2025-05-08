script;

struct S {}
enum E { None: (), Some: S }

fn main() -> u64 {
    // Primitive types
    let _ = __dbg(8u8);
    let _ = __dbg(16u16);
    let _ = __dbg(32u32);
    let _ = __dbg(64u64);
    let _ = __dbg(0x100u256);

    // strings
    let _ = __dbg("Hello!");
    let _ = __dbg(__to_str_array("Hello!"));

    // Aggregates
    let _ = __dbg((1u64, 2u64));
    let _ = __dbg([1u64, 2u64]);

    // Strucs and Enum
    let _ = __dbg(S { });
    let _ = __dbg(E::None);
    let _ = __dbg(E::Some(S { }));

    // should return its argument
    __dbg(11u64)
}