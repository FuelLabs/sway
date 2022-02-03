.program:
ji   i4
noop
DATA_SECTION_OFFSET[0..32]
DATA_SECTION_OFFSET[32..64]
lw   $ds $is 1
add  $$ds $$ds $is
move $r0 $sp                  ; save locals base register
cfei i32                      ; allocate 32 bytes for all locals
lw   $r1 data_0               ; literal instantiation
addi $r2 $r0 i0               ; get store offset
mcpi $r2 $r1 i32              ; store value
addi $r1 $r0 i0               ; load address
lw   $r0 data_1               ; literal instantiation
addi $r2 $zero i32            ; asm block
meq  $r3 $r1 $r0 $r2          ; asm block
move $r0 $r3                  ; return value from inline asm
move $r1 $r0
ret  $r1
noop                          ; word-alignment of data section
.data:
data_0 .b256 0x0202020202020202020202020202020202020202020202020202020202020202
data_1 .b256 0x0303030303030303030303030303030303030303030303030303030303030303
