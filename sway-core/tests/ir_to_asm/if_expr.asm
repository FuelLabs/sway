.program:
ji   i4
noop
DATA_SECTION_OFFSET[0..32]
DATA_SECTION_OFFSET[32..64]
lw   $ds $is 1
add  $$ds $$ds $is
lw   $r0 data_0               ; literal instantiation
jnzi $r0 i9
ji   i11
lw   $r0 data_1               ; literal instantiation
ji   i12
lw   $r0 data_2               ; literal instantiation
ret  $r0
.data:
data_0 .bool 0x00
data_1 .u64 0xf4240
data_2 .u64 0x2a
