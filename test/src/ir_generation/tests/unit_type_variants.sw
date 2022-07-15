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
// nextln: entry:
// nextln: $(enum_undef=$VAL) = const { u64 } { u64 undef }
// nextln: $(two=$VAL) = const u64 2
// nextln: $(enum=$VAL) = insert_value $enum_undef, { u64 }, $two, 0
// nextln: ret { u64 } $enum
