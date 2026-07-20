script;

// Regression test for SROA miscompilations.
//
// A reference to a *non-first* element of a local tuple is chosen inside a
// conditional. In IR, the chosen reference reaches the join point as a block argument
// and is only dereferenced there. The pointer carried by a block
// argument is not a plain GEP into `t`, so **its offset cannot be determined by
// `combine_indices`**. SROA must therefore **not scalarise `t`**.
//
// A previous bug treated the block-argument pointer in `combine_indices` as offset 0,
// so `*r` incorrectly always read `t.0` (10) regardless of which element was referenced.
//
// This test was passing in `debug` mode, because the SROA pass wasn't executed.
// In `release` mode it was failing with always returning the element at index 0.
//
// With `sel == 0` the result must be `t.1` (20), not `t.0` (10).
//
// Additionally, even if the above bug in `combine_indices` is removed,
// furthermore SROA didn't remove candidates whose `combine_indices` was `None`
// (those coming from unknown sources, with unknown previous indices).
//
// This test ensures that both the issues are fixed.

fn main(sel: u64) -> u64 {
    let t = (10u64, 20u64, 30u64);
    let r = if sel == 0 { &t.1 } else { &t.2 };
    *r
}
