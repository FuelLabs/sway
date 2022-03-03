.program:
ji   i4
noop
DATA_SECTION_OFFSET[0..32]
DATA_SECTION_OFFSET[32..64]
lw   $ds $is 1
add  $$ds $$ds $is
move $r0 $sp                  ; save locals base register
lw   $r0 data_0               ; literal instantiation
jnei $r0 $one i12
jnei $r0 $one i11
lw   $r0 data_1               ; literal instantiation
ji   i8
ret  $r0
.data:
data_0 .bool 0x01
data_1 .bool 0x00
