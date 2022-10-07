.program:
ji   i4
noop
DATA_SECTION_OFFSET[0..32]
DATA_SECTION_OFFSET[32..64]
lw   $ds $is 1
add  $$ds $$ds $is
move $r2 $sp                  ; save locals base register
cfei i8                       ; allocate 8 bytes for locals
addi $r0 $r2 i0               ; get offset reg for get_ptr
sw   $r2 $one i0              ; store value
addi $r0 $r2 i0               ; get offset reg for get_ptr
lw   $r0 $r2 i0               ; load value
jnzi $r0 i14
ji   i22
addi $r0 $r2 i0               ; get offset reg for get_ptr
lw   $r0 $r2 i0               ; load value
jnzi $r0 i18
ji   i19
move $r0 $zero                ; branch to phi value
addi $r1 $r2 i0               ; get offset reg for get_ptr
sw   $r2 $r0 i0               ; store value
ji   i10
addi $r0 $r2 i0               ; get offset reg for get_ptr
lw   $r0 $r2 i0               ; load value
ret  $r0
.data:
