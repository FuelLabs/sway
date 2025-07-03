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

// check: $(temp_ptr=$VAL) = get_local __ptr { u64 }, $(=__anon_\d+)
// check: $(idx_0=$VAL) = const u64 0
// nextln: $(tag_ptr=$VAL) = get_elem_ptr $temp_ptr, __ptr u64, $idx_0
// nextln: $(tag_2=$VAL) = const u64 2
// nextln: store $tag_2 to $tag_ptr
// nextln: $(temp_val=$VAL) = load $temp_ptr
// nextln: ret { u64 } $temp_val
