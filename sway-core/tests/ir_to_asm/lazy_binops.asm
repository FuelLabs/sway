.program:
ji   i4
noop
DATA_SECTION_OFFSET[0..32]
DATA_SECTION_OFFSET[32..64]
lw   $ds $is 1
add  $$ds $$ds $is
lw   $r0 data_0               ; literal instantiation
move $r1 $r0
move $r1 $r0
jnei $r0 $one i12
lw   $r0 data_1               ; literal instantiation
move $r1 $r0
move $r0 $r1
move $r2 $r1
jnei $r1 $one i16
ji   i18
lw   $r1 data_1               ; literal instantiation
move $r0 $r1
ret  $r0
.data:
data_0 .bool 0x00
data_1 .bool 0x01
