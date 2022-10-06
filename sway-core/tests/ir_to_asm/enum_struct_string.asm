.program:
ji   i4
noop
DATA_SECTION_OFFSET[0..32]
DATA_SECTION_OFFSET[32..64]
lw   $ds $is 1
add  $$ds $$ds $is
move $r4 $sp                  ; save locals base register
cfei i48                      ; allocate 48 bytes for locals
lw   $r3 data_0               ; literal instantiation
sw   $r3 $zero i0             ; insert_value @ 0
lw   $r2 data_0               ; literal instantiation
lw   $r1 data_1               ; literal instantiation
addi $r0 $r2 i0               ; get struct field(s) 0 offset
mcpi $r0 $r1 i24              ; store struct field value
lw   $r0 data_2               ; literal instantiation
sw   $r2 $r0 i3               ; insert_value @ 1
lw   $r1 data_3               ; literal instantiation
addi $r0 $r1 i0               ; get struct field(s) 0 offset
mcpi $r0 $r2 i32              ; store struct field value
lw   $r0 data_4               ; literal instantiation
sw   $r1 $r0 i4               ; insert_value @ 1
sw   $r1 $zero i5             ; insert_value @ 2
addi $r0 $r3 i8               ; get struct field(s) 1 offset
mcpi $r0 $r1 i48              ; store struct field value
lw   $r0 $r3 i0               ; extract_value @ 0
eq   $r0 $r0 $zero
jnzi $r0 i32
ji   i39
addi $r1 $r3 i8               ; extract address
addi $r0 $r4 i0               ; get offset reg for get_ptr
addi $r0 $r4 i0               ; get store offset
mcpi $r0 $r1 i48              ; store value
addi $r0 $r4 i0               ; get offset reg for get_ptr
lw   $r0 $r0 i4               ; extract_value @ 1
ji   i40
move $r0 $zero                ; parameter from branch to block argument
ret  $r0
.data:
data_0 .collection { .word 0, .word 0 }
data_1 .bytes[17] ee 82 b0 20 61 6e 20 6f 64 64 20 6c 65 6e 67 74 68  ... an odd length
data_2 .word 20
data_3 .collection { .collection { .word 0, .word 0 }, .word 0, .word 0 }
data_4 .word 10
