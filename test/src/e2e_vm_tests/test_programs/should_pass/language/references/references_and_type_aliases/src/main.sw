script;

type RefToU64 = &u64;
// TODO-IG: Once https://github.com/FuelLabs/sway/issues/5401 is solved use the line below.
type RefToTupleOfRefs = &(&u64, &u64);
//type RefToTupleOfRefs = &(&u64, RefToU64);

fn references_and_type_aliases() {
    let r: RefToU64 = &123;
    let t: RefToTupleOfRefs = &(r, r);

    let ret: &u64 = passing_and_returning_ref_type_aliases(t);

    let ret_ptr = asm(r: ret) { r: raw_ptr };

    assert(ret_ptr.read::<u64>() == 123);
}

fn passing_and_returning_ref_type_aliases(x: RefToTupleOfRefs) -> RefToU64 {
    let x: &(&u64, &u64) = x;

    let ptr = asm(r: x) { r: raw_ptr };

    let tuple = ptr.read::<(RefToU64, RefToU64)>();

    tuple.0
}

fn main() -> u64 {
    references_and_type_aliases();
    
    42
}
