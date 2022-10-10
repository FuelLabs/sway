.program:
ji   i4
noop
DATA_SECTION_OFFSET[0..32]
DATA_SECTION_OFFSET[32..64]
lw   $ds $is 1
add  $$ds $$ds $is
move $r2 $sp                  ; save locals base register
cfei i24                      ; allocate 24 bytes for locals
lw   $r1 data_0               ; literal instantiation
muli $r0 $zero i8             ; insert_element relative offset
add  $r0 $r1 $r0              ; insert_element absolute offset
sw   $r0 $zero i0             ; insert_element
muli $r0 $one i8              ; insert_element relative offset
add  $r0 $r1 $r0              ; insert_element absolute offset
sw   $r0 $one i0              ; insert_element
lw   $r0 data_1               ; literal instantiation
muli $r0 $r0 i8               ; insert_element relative offset
add  $r0 $r1 $r0              ; insert_element absolute offset
sw   $r0 $zero i0             ; insert_element
addi $r0 $r2 i0               ; get offset reg for get_ptr
addi $r0 $r2 i0               ; get store offset
mcpi $r0 $r1 i24              ; store value
addi $r1 $r2 i0               ; get offset reg for get_ptr
muli $r0 $one i8              ; extract_element relative offset
add  $r0 $r1 $r0              ; extract_element absolute offset
lw   $r0 $r0 i0               ; extract_element
ret  $r0
.data:
data_0 .collection { .word 0, .word 0, .word 0 }
data_1 .word 2
