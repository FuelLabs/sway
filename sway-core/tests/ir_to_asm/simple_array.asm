.program:
ji   i4
noop
DATA_SECTION_OFFSET[0..32]
DATA_SECTION_OFFSET[32..64]
lw   $ds $is 1
add  $$ds $$ds $is
move $r3 $sp                  ; save locals base register
cfei i24                      ; allocate 24 bytes for all locals
move $r2 $sp                  ; save register for temporary stack value
cfei i24                      ; allocate 24 bytes for temporary array
lw   $r1 data_0               ; literal instantiation
lw   $r0 data_1               ; literal instantiation
muli $r0 $r0 i8               ; insert_element relative offset
add  $r0 $r2 $r0              ; insert_element absolute offset
sw   $r0 $r1 i0               ; insert_element
lw   $r1 data_2               ; literal instantiation
lw   $r0 data_3               ; literal instantiation
muli $r0 $r0 i8               ; insert_element relative offset
add  $r0 $r2 $r0              ; insert_element absolute offset
sw   $r0 $r1 i0               ; insert_element
lw   $r1 data_0               ; literal instantiation
lw   $r0 data_4               ; literal instantiation
muli $r0 $r0 i8               ; insert_element relative offset
add  $r0 $r2 $r0              ; insert_element absolute offset
sw   $r0 $r1 i0               ; insert_element
addi $r0 $r3 i0               ; get store offset
mcpi $r0 $r2 i24              ; store value
addi $r1 $r3 i0               ; load address
lw   $r0 data_3               ; literal instantiation
muli $r0 $r0 i8               ; extract_element relative offset
add  $r0 $r1 $r0              ; extract_element absolute offset
lw   $r0 $r0 i0               ; extract_element
ret  $r0
.data:
data_0 .bool 0x00
data_1 .u64 0x00
data_2 .bool 0x01
data_3 .u64 0x01
data_4 .u64 0x02
