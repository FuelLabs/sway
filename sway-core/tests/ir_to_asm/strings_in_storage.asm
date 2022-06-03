.program:
ji   i4
noop
DATA_SECTION_OFFSET[0..32]
DATA_SECTION_OFFSET[32..64]
lw   $ds $is 1
add  $$ds $$ds $is
lw   $r1 $fp i73              ; load input function selector
lw   $r0 data_3               ; load fn selector for comparison
eq   $r0 $r1 $r0              ; function selector comparison
jnzi $r0 i14                  ; jump to selected function
lw   $r0 data_4               ; load fn selector for comparison
eq   $r0 $r1 $r0              ; function selector comparison
jnzi $r0 i39                  ; jump to selected function
rvrt $zero                    ; revert if no selectors matched
move $r3 $sp                  ; save locals base register
cfei i96                      ; allocate 96 bytes for all locals
lw   $r2 $fp i74              ; Base register for method parameter
addi $r0 $r3 i0               ; get offset reg for get_ptr
lw   $r1 data_0               ; literal instantiation
addi $r0 $r3 i0               ; get store offset
mcpi $r0 $r1 i32              ; store value
addi $r0 $r3 i32              ; get offset reg for get_ptr
addi $r0 $r3 i32              ; get store offset
mcpi $r0 $r2 i64              ; store value
addi $r0 $r3 i32              ; get offset reg for get_ptr
addi $r1 $r3 i32              ; get offset
addi $r0 $r3 i0               ; get offset
swwq $r0 $r1                  ; quad word state access
addi $r0 $r3 i0               ; get offset reg for get_ptr
lw   $r1 data_1               ; literal instantiation
addi $r0 $r3 i0               ; get store offset
mcpi $r0 $r1 i32              ; store value
addi $r0 $r3 i64              ; get offset reg for get_ptr
addi $r1 $r3 i64              ; get offset
addi $r0 $r3 i0               ; get offset
swwq $r0 $r1                  ; quad word state access
ret  $zero                    ; returning unit as zero
move $r3 $sp                  ; save locals base register
cfei i96                      ; allocate 96 bytes for all locals
addi $r0 $r3 i0               ; get offset reg for get_ptr
lw   $r1 data_0               ; literal instantiation
addi $r0 $r3 i0               ; get store offset
mcpi $r0 $r1 i32              ; store value
addi $r2 $r3 i32              ; get offset reg for get_ptr
addi $r0 $r3 i32              ; get offset reg for get_ptr
addi $r1 $r3 i32              ; get offset
addi $r0 $r3 i0               ; get offset
srwq $r1 $r0                  ; quad word state access
addi $r0 $r3 i0               ; get offset reg for get_ptr
lw   $r1 data_1               ; literal instantiation
addi $r0 $r3 i0               ; get store offset
mcpi $r0 $r1 i32              ; store value
addi $r0 $r3 i64              ; get offset reg for get_ptr
addi $r1 $r3 i64              ; get offset
addi $r0 $r3 i0               ; get offset
srwq $r1 $r0                  ; quad word state access
lw   $r0 data_2               ; loading size for RETD
retd  $r2 $r0
noop                          ; word-alignment of data section
.data:
data_0 .b256 0xf383b0ce51358be57daa3b725fe44acdb2d880604e367199080b4379c41bb6ed
data_1 .b256 0xf383b0ce51358be57daa3b725fe44acdb2d880604e367199080b4379c41bb6ee
data_2 .u64 0x28
data_3 .u32 0xe63a9733
data_4 .u32 0xb8c27db9
