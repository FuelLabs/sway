.program:
ji   i4
noop
DATA_SECTION_OFFSET[0..32]
DATA_SECTION_OFFSET[32..64]
lw   $ds $is 1
add  $$ds $$ds $is
move $r0 $sp                  ; save locals base register
cfei i16                      ; allocate 16 bytes for all locals
move $r1 $sp                  ; save register for temporary stack value
cfei i16                      ; allocate 16 bytes for temporary struct
lw   $r2 data_0               ; literal instantiation
sw   $r1 $r2 i0               ; insert_value @ 0
addi $r2 $r0 i0               ; get store offset
mcpi $r2 $r1 i16              ; store value
addi $r1 $r0 i0               ; get_ptr
lw   $r0 data_1               ; literal instantiation
move $r1 $r0
move $r0 $sp                  ; save register for temporary stack value
cfei i16                      ; allocate 16 bytes for temporary struct
lw   $r1 data_2               ; literal instantiation
sw   $r0 $r1 i0               ; insert_value @ 0
lw   $r1 data_3               ; literal instantiation
sw   $r0 $r1 i1               ; insert_value @ 1
lw   $r0 data_1               ; literal instantiation
move $r1 $r0
ret  $zero                    ; returning unit as zero
noop                          ; word-alignment of data section
.data:
data_0 .u64 0x01
data_1 .bool 0x00
data_2 .u64 0x02
data_3 .u64 0x03
