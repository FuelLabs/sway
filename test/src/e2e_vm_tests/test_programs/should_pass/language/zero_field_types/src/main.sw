script;

struct StructWithNoFields {}
enum EnumWithNoVariants {}

fn main() -> u64 {
    let unit_struct = StructWithNoFields {};
    10u64
}
