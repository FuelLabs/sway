.program:
ji   i4
noop
DATA_SECTION_OFFSET[0..32]
DATA_SECTION_OFFSET[32..64]
lw   $ds $is 1
add  $$ds $$ds $is
jnzi $zero i8
ji   i10
lw   $r0 data_0               ; literal instantiation
ji   i11
lw   $r0 data_1               ; literal instantiation
ret  $r0
noop                          ; word-alignment of data section
.data:
data_0 .u64 0xf4240
data_1 .u64 0x2a
