script;

struct StructWithNoFields {}
enum EnumWithNoVariants {}

fn main() -> u64 {
    let _unit_struct = StructWithNoFields {};
    10u64
}
