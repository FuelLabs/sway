.program:
ji   i4
noop
DATA_SECTION_OFFSET[0..32]
DATA_SECTION_OFFSET[32..64]
lw   $ds $is 1
add  $$ds $$ds $is
move $r2 $sp                  ; save locals base register
cfei i32                      ; allocate 32 bytes for all locals
lw   $r1 data_0               ; literal instantiation
addi $r0 $r2 i0               ; get store offset
mcpi $r0 $r1 i32              ; store value
addi $r2 $r2 i0               ; load address
lw   $r1 data_1               ; literal instantiation
addi $r0 $zero i32            ; asm block
meq  $r0 $r2 $r1 $r0          ; asm block
ret  $r0
noop                          ; word-alignment of data section
.data:
data_0 .b256 0x0202020202020202020202020202020202020202020202020202020202020202
data_1 .b256 0x0303030303030303030303030303030303030303030303030303030303030303
