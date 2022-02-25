.program:
ji   i4
noop
DATA_SECTION_OFFSET[0..32]
DATA_SECTION_OFFSET[32..64]
lw   $ds $is 1
add  $$ds $$ds $is
lw   $r0 data_0               ; literal instantiation
jnei $r0 $one i11
jnei $r0 $one i10
lw   $r0 data_1               ; literal instantiation
ji   i7
ret  $r0
noop                          ; word-alignment of data section
.data:
data_0 .bool 0x01
data_1 .bool 0x00
