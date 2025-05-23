predicate;

fn main(x: u64, y: u64) -> bool {
    true
}
// ::check-ir::

// check: predicate {

// ::check-asm::

// regex: IMM=i\d+
// regex: REG=\$r\d+

// check: gm   $(r2=$REG) i3
// check: gtf  $(r1=$REG) $r2 i512
// nextln: jnzf $r1 $$zero $IMM
// check: jmpf $$zero $IMM
// check: movi $(r0=$REG) i2
// check: eq   $r1 $r1 $r0
// check: xori $r1 $r1 i1
// check: jnzf $r1 $$zero $IMM
// check: jmpf $$zero $IMM
// check: ret  $$zero
