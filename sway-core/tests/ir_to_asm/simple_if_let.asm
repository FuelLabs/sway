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
sw   $r1 $zero i0             ; insert_value @ 0
sw   $r1 $one i1              ; insert_value @ 1
addi $r0 $r2 i8               ; get offset reg for get_ptr
addi $r0 $r2 i8               ; get store offset
mcpi $r0 $r1 i16              ; store value
addi $r1 $r2 i8               ; get offset reg for get_ptr
lw   $r0 $r1 i0               ; extract_value @ 0
eq   $r0 $r0 $one
jnzi $r0 i20
ji   i26
lw   $r1 $r1 i1               ; extract_value @ 1,1
addi $r0 $r2 i0               ; get offset reg for get_ptr
sw   $r2 $r1 i0               ; store value
addi $r0 $r2 i0               ; get offset reg for get_ptr
lw   $r0 $r2 i0               ; load value
ji   i27
move $r0 $zero                ; branch to phi value
ret  $r0
.data:
data_0 .collection { .word 0, .word 0 }
