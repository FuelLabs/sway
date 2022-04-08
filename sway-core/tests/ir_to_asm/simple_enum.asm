.program:
ji   i4
noop
DATA_SECTION_OFFSET[0..32]
DATA_SECTION_OFFSET[32..64]
lw   $ds $is 1
add  $$ds $$ds $is
move $r2 $sp                  ; save locals base register
cfei i16                      ; allocate 16 bytes for all locals
move $r1 $sp                  ; save register for temporary stack value
cfei i16                      ; allocate 16 bytes for temporary struct
lw   $r0 data_0               ; literal instantiation
sw   $r1 $r0 i0               ; insert_value @ 0
addi $r0 $r2 i0               ; get_ptr
addi $r0 $r2 i0               ; get store offset
mcpi $r0 $r1 i16              ; store value
addi $r0 $r2 i0               ; get_ptr
move $r2 $sp                  ; save register for temporary stack value
cfei i16                      ; allocate 16 bytes for temporary struct
lw   $r0 data_1               ; literal instantiation
sw   $r2 $r0 i0               ; insert_value @ 0
lw   $r1 data_2               ; literal instantiation
addi $r0 $r2 i8               ; get struct field(s) 1 offset
mcpi $r0 $r1 i8               ; store struct field value
ret  $zero                    ; returning unit as zero
noop                          ; word-alignment of data section
.data:
data_0 .u64 0x01
data_1 .u64 0x02
data_2 .u64 0x03
