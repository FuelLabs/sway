predicate;

fn main(x: u64) -> bool {
    true
}
// ::check-ir::

// check: predicate {

// ::check-asm::

// regex: IMM=i\d+
// regex: REG=\$r\d+

// check: gtf  $(r1=$REG) $$zero i257
// nextln: jnzi $r1 $IMM
// nextln: gtf  $(r2=$REG) $$zero i269
// nextln: ji   $IMM
// nextln: movi $(r0=$REG) i2
// nextln: eq   $r0 $r1 $r0
// nextln: jnzi $r0 $IMM
// nextln: gtf  $r2 $$zero i288
// nextln: ji   $IMM
// nextln: ret  $$zero
// nextln: lw   $r2 $r2 i0
