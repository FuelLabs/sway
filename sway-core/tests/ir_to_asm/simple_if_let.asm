.program:
ji   i4
noop
DATA_SECTION_OFFSET[0..32]
DATA_SECTION_OFFSET[32..64]
lw   $ds $is 1
add  $$ds $$ds $is
move $r3 $sp                  ; save locals base register
cfei i24                      ; allocate 24 bytes for all locals
move $r1 $sp                  ; save register for temporary stack value
cfei i16                      ; allocate 16 bytes for temporary struct
lw   $r0 data_0               ; literal instantiation
sw   $r1 $r0 i0               ; insert_value @ 0
lw   $r0 data_1               ; literal instantiation
sw   $r1 $r0 i1               ; insert_value @ 1
addi $r0 $r3 i8               ; get_ptr
addi $r0 $r3 i8               ; get store offset
mcpi $r0 $r1 i16              ; store value
addi $r2 $r3 i8               ; get_ptr
lw   $r1 $r2 i0               ; extract_value @ 0
lw   $r0 data_2               ; literal instantiation
eq   $r0 $r1 $r0
jnzi $r0 i23
ji   i29
lw   $r1 $r2 i1               ; extract_value @ 1,1
addi $r0 $r3 i0               ; get_ptr
sw   $r3 $r1 i0               ; store value
addi $r0 $r3 i0               ; get_ptr
lw   $r0 $r3 i0               ; load value
ji   i30
lw   $r0 data_0               ; literal instantiation
ret  $r0
.data:
data_0 .u64 0x00
data_1 .bool 0x01
data_2 .u64 0x01
