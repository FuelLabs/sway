.program:
ji   i4
noop
DATA_SECTION_OFFSET[0..32]
DATA_SECTION_OFFSET[32..64]
lw   $ds $is 1
add  $$ds $$ds $is
move $r0 $zero                ; branch to phi value
move $r0 $zero                ; branch to phi value
jnzi $zero i10
ji   i11
move $r0 $one                 ; branch to phi value
move $r1 $r0                  ; branch to phi value
jnzi $r0 i14
move $r0 $one                 ; branch to phi value
ret  $r0
.data:
