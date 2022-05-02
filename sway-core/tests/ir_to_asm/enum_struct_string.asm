.program:
ji   i4
noop
DATA_SECTION_OFFSET[0..32]
DATA_SECTION_OFFSET[32..64]
lw   $ds $is 1
add  $$ds $$ds $is
move $r4 $sp                  ; save locals base register
cfei i48                      ; allocate 48 bytes for all locals
move $r3 $sp                  ; save register for temporary stack value
cfei i56                      ; allocate 56 bytes for temporary struct
lw   $r0 data_0               ; literal instantiation
sw   $r3 $r0 i0               ; insert_value @ 0
move $r2 $sp                  ; save register for temporary stack value
cfei i32                      ; allocate 32 bytes for temporary struct
lw   $r1 data_1               ; literal instantiation
addi $r0 $r2 i0               ; get struct field(s) 0 offset
mcpi $r0 $r1 i24              ; store struct field value
lw   $r0 data_2               ; literal instantiation
sw   $r2 $r0 i3               ; insert_value @ 1
move $r1 $sp                  ; save register for temporary stack value
cfei i48                      ; allocate 48 bytes for temporary struct
addi $r0 $r1 i0               ; get struct field(s) 0 offset
mcpi $r0 $r2 i32              ; store struct field value
lw   $r0 data_3               ; literal instantiation
sw   $r1 $r0 i4               ; insert_value @ 1
lw   $r0 data_4               ; literal instantiation
sw   $r1 $r0 i5               ; insert_value @ 2
addi $r0 $r3 i8               ; get struct field(s) 1 offset
mcpi $r0 $r1 i48              ; store struct field value
lw   $r1 $r3 i0               ; extract_value @ 0
lw   $r0 data_0               ; literal instantiation
eq   $r0 $r1 $r0
jnzi $r0 i35
ji   i42
addi $r1 $r3 i8               ; extract address
addi $r0 $r4 i0               ; get_ptr
addi $r0 $r4 i0               ; get store offset
mcpi $r0 $r1 i48              ; store value
addi $r0 $r4 i0               ; get_ptr
lw   $r0 $r0 i4               ; extract_value @ 1
ji   i43
lw   $r0 data_0               ; literal instantiation
ret  $r0
.data:
data_0 .u64 0x00
data_1 .str " an odd length"
data_2 .u64 0x14
data_3 .u64 0x0a
data_4 .bool 0x00
