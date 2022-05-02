.program:
ji   i4
noop
DATA_SECTION_OFFSET[0..32]
DATA_SECTION_OFFSET[32..64]
lw   $ds $is 1
add  $$ds $$ds $is
move $r2 $sp                  ; save locals base register
cfei i8                       ; allocate 8 bytes for all locals
addi $r0 $r2 i0               ; get_ptr
lw   $r0 data_0               ; literal instantiation
sw   $r2 $r0 i0               ; store value
addi $r0 $r2 i0               ; get_ptr
lw   $r0 $r2 i0               ; load value
jnzi $r0 i15
ji   i23
addi $r0 $r2 i0               ; get_ptr
lw   $r0 $r2 i0               ; load value
jnzi $r0 i19
ji   i20
lw   $r0 data_1               ; literal instantiation
addi $r1 $r2 i0               ; get_ptr
sw   $r2 $r0 i0               ; store value
ji   i11
addi $r0 $r2 i0               ; get_ptr
lw   $r0 $r2 i0               ; load value
ret  $r0
noop                          ; word-alignment of data section
.data:
data_0 .bool 0x01
data_1 .bool 0x00
