.program:
ji   i4
noop
DATA_SECTION_OFFSET[0..32]
DATA_SECTION_OFFSET[32..64]
lw   $ds $is 1
add  $$ds $$ds $is
lw   $r0 data_0               ; literal instantiation
move $r1 $r0
move $r0 $r1
jnei $r0 $one i18
move $r0 $r1
move $r2 $r0
move $r2 $r0
jnei $r0 $one i16
lw   $r0 data_1               ; literal instantiation
move $r2 $r0
move $r1 $r2
ji   i8
move $r0 $r1
ret  $r0
noop                          ; word-alignment of data section
.data:
data_0 .bool 0x01
data_1 .bool 0x00
