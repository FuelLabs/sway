.program:
ji   i4
noop
DATA_SECTION_OFFSET[0..32]
DATA_SECTION_OFFSET[32..64]
lw   $ds $is 1
add  $$ds $$ds $is
lw   $r0 data_0               ; literal instantiation
lw   $r1 data_0               ; literal instantiation
lw   $r0 data_0               ; literal instantiation
jnei $r0 $one i11
lw   $r1 data_1               ; literal instantiation
move $r0 $r1
jnei $r1 $one i14
ji   i15
lw   $r1 data_1               ; literal instantiation
ret  $r1
noop                          ; word-alignment of data section
.data:
data_0 .bool 0x00
data_1 .bool 0x01
