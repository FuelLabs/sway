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
// nextln: jnzi $r1 $IMM
// nextln: gtf  $(r3=$REG) $r2 i269
// nextln: ji   $IMM
// nextln: movi $(r0=$REG) i2
// nextln: eq   $r0 $r1 $r0
// nextln: not  $r0 $r0
// nextln: jnzi $r0 $IMM
// nextln: gtf  $r3 $r2 i288
// nextln: ji   $IMM
// nextln: ret  $$zero
// nextln: lw   $r0 $r3 i0
// nextln: lw   $r0 $r3 i1
