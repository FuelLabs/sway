.program:
ji   i4
noop
DATA_SECTION_OFFSET[0..32]
DATA_SECTION_OFFSET[32..64]
lw   $ds $is 1
add  $$ds $$ds $is
bhei $r0                      ; asm block
move $r1 $r0                  ; return value from inline asm
ret  $r1
.data:
