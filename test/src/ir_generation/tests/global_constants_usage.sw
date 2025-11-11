// target-fuelvm
script;

const ADDRESS: b256 = 0x9999999999999999999999999999999999999999999999999999999999999999;

fn main() {
    poke(ADDRESS);
}

#[inline(never)]
fn poke<T>(_t: T) { }

// ::check-ir::

// check: fn main() -> ()

// ::check-ir-optimized::
// pass: o1

// check: $(v0=$VAL) = get_global __ptr b256, test_lib::ADDRESS
// nextln: $(v1=$VAL) = get_local __ptr b256, __tmp_arg
// nextln: mem_copy_val $v1, $v0
// nextln: $(v2=$VAL) = call poke_0($v1)
