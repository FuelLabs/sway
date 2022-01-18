.program:
ji   i4
noop
DATA_SECTION_OFFSET[0..32]
DATA_SECTION_OFFSET[32..64]
lw   $ds $is 1
add  $$ds $$ds $is
move $r0 $sp
cfei i16
move $r1 $sp
cfei i16
lw   $r2 data_0               ; literal instantiation
sw   $r1 $r2 i0               ; insert_value @ 0
lw   $r2 data_1               ; literal instantiation
sw   $r1 $r2 i1               ; insert_value @ 1
addi $r2 $r0 i0               ; store get offset
mcpi $r2 $r1 i16              ; store value
addi $r1 $r0 i0               ; get_ptr
lw   $r2 data_2               ; literal instantiation
sw   $r1 $r2 i0               ; insert_value @ 0
addi $r1 $r0 i0               ; get_ptr
lw   $r0 $r1 i1               ; extract_value @ 1
ret  $r0
noop                          ; word-alignment of data section
.data:
data_0 .u64 0x28
data_1 .u64 0x02
data_2 .u64 0x32
