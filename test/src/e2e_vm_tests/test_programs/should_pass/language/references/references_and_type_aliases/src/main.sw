script;

type RefToU64 = &u64;
// TODO: (REFERENCES) Once https://github.com/FuelLabs/sway/issues/5401 is solved use the line below.
type RefToTupleOfRefs = &(&u64, &u64);
//type RefToTupleOfRefs = &(&u64, RefToU64);
type RefToMutU64 = &mut u64;
type RefToMutTupleOfMutRefs = &mut (&mut u64, &mut u64);

fn references_and_type_aliases() {
    let r: RefToU64 = &123;
    let t: RefToTupleOfRefs = &(r, r);
    let t_mut: RefToTupleOfRefs = &mut (&mut 123, &mut 123);

    let ret: (&u64, &mut u64) = passing_and_returning_ref_type_aliases(t, t_mut);

    let ret_0_ptr = asm(r: ret.0) { r: raw_ptr };
    let ret_1_ptr = asm(r: ret.1) { r: raw_ptr };

    assert(ret_0_ptr.read::<u64>() == 123);
    assert(ret_1_ptr.read::<u64>() == 123);

    assert(*r == 123);
    assert(*t.0 == 123);
    assert(*t.1 == 123);
}

fn passing_and_returning_ref_type_aliases(x: RefToTupleOfRefs, y: RefToMutTupleOfMutRefs) -> (RefToU64, RefToMutU64) {
    let x: &(&u64, &u64) = x;
    let y: &mut (&mut u64, &mut u64) = y;

    let ptr_x = asm(r: x) { r: raw_ptr };
    let ptr_y = asm(r: y) { r: raw_ptr };

    let tuple_x = ptr_x.read::<(RefToU64, RefToU64)>();
    let tuple_y = ptr_y.read::<(RefToMutU64, RefToMutU64)>();

    (tuple_x.0, tuple_y.0)
}

fn main() -> u64 {
    references_and_type_aliases();
    
    42
}
