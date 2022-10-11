.program:
ji   i4
noop
DATA_SECTION_OFFSET[0..32]
DATA_SECTION_OFFSET[32..64]
lw   $ds $is 1
add  $$ds $$ds $is
move $r0 $zero                ; parameter from branch to block argument
jnzi $zero i9
ji   i10
move $r0 $one                 ; parameter from branch to block argument
jnzi $r0 i12
move $r0 $one                 ; parameter from branch to block argument
ret  $r0
.data:
