.program:
ji   i4
noop
DATA_SECTION_OFFSET[0..32]
DATA_SECTION_OFFSET[32..64]
lw   $ds $is 1
add  $$ds $$ds $is
move $r3 $sp                  ; save locals base register
cfei i8                       ; allocate 8 bytes for all locals
move $r0 $sp                  ; save register for temporary stack value
cfei i8                       ; allocate 8 bytes for temporary struct
lw   $r1 data_0               ; literal instantiation for aggregate field
sw   $r0 $r1 i0               ; initialise aggregate field
move $r2 $sp                  ; save register for temporary stack value
cfei i16                      ; allocate 16 bytes for temporary struct
lw   $r1 data_1               ; literal instantiation for aggregate field
sw   $r2 $r1 i0               ; initialise aggregate field
addi $r1 $r2 i8               ; get struct field(s) 1 offset
mcpi $r1 $r0 i8               ; store struct field value
lw   $r1 $r2 i0               ; extract_value @ 0
lw   $r0 data_1               ; literal instantiation
eq   $r0 $r1 $r0
jnzi $r0 i23
ji   i30
addi $r1 $r2 i8               ; extract address
addi $r0 $r3 i0               ; get offset reg for get_ptr
addi $r0 $r3 i0               ; get store offset
mcpi $r0 $r1 i8               ; store value
addi $r0 $r3 i0               ; get offset reg for get_ptr
lw   $r0 $r0 i0               ; extract_value @ 0
ji   i31
lw   $r0 data_2               ; literal instantiation
ret  $r0
noop                          ; word-alignment of data section
.data:
data_0 .u64 0x2a
data_1 .u64 0x01
data_2 .u64 0x00
