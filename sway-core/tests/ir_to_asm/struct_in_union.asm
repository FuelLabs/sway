.program:
ji   i4
noop
DATA_SECTION_OFFSET[0..32]
DATA_SECTION_OFFSET[32..64]
lw   $ds $is 1
add  $$ds $$ds $is
move $r3 $sp                  ; save locals base register
cfei i8                       ; allocate 8 bytes for locals
lw   $r0 data_0               ; literal instantiation
lw   $r2 data_1               ; literal instantiation
addi $r1 $r2 i8               ; get struct field(s) 1 offset
mcpi $r1 $r0 i8               ; store struct field value
lw   $r0 $r2 i0               ; extract_value @ 0
eq   $r0 $r0 $one
jnzi $r0 i18
ji   i25
addi $r1 $r2 i8               ; extract address
addi $r0 $r3 i0               ; get offset reg for get_ptr
addi $r0 $r3 i0               ; get store offset
mcpi $r0 $r1 i8               ; store value
addi $r0 $r3 i0               ; get offset reg for get_ptr
lw   $r0 $r0 i0               ; extract_value @ 0
ji   i26
move $r0 $zero                ; parameter from branch to block argument
ret  $r0
.data:
data_0 .collection { .word 42 }
data_1 .collection { .word 1, .word 0 }
