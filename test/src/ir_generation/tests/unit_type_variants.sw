script;

enum E {
    A: (),
    B: (),
    C: (),
}

fn main() -> E {
    E::C
}

// Since all variants are unit the tagged union has no value, it's just a tag.

// check: fn main() -> { u64 }
// check: entry():
// nextln: $(enum_undef=$VAL) = get_ptr ptr { u64 } $ID, ptr { u64 }, 0
// nextln: $(two=$VAL) = const u64 2
// nextln: $(enum=$VAL) = insert_value $enum_undef, { u64 }, $two, 0
// nextln: ret { u64 } $enum
