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
// check: gtf  $(r1=$REG) $r2 i257
// nextln: jnzf $r1 $$zero $IMM
// nextln: gtf  $(r3=$REG) $r2 i269
// nextln: jmpf $$zero $IMM
// nextln: movi $(r0=$REG) i2
// nextln: eq   $r1 $r1 $r0
// nextln: xori $r1 $r1 i1
// nextln: jnzf $r1 $$zero $IMM
// nextln: gtf  $r3 $r2 i287
// nextln: jmpf $$zero $IMM
// nextln: ret  $$zero
// nextln: lw   $r2 $r3 i0
// nextln: lw   $r3 $r3 i1
