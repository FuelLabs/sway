.program:
ji   i4
noop
DATA_SECTION_OFFSET[0..32]
DATA_SECTION_OFFSET[32..64]
lw   $ds $is 1
add  $$ds $$ds $is
lw   $r1 data_0               ; literal instantiation
lw   $r0 data_1               ; loading size for RETD
retd  $r1 $r0
.data:
data_0 .collection { .word 1, .collection { .word 42, .collection { .word 1, .word 66 } } }
data_1 .word 72
