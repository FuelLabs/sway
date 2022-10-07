.program:
ji   i4
noop
DATA_SECTION_OFFSET[0..32]
DATA_SECTION_OFFSET[32..64]
lw   $ds $is 1
add  $$ds $$ds $is
lw   $r0 $fp i73              ; load input function selector
lw   $r1 data_3               ; load fn selector for comparison
eq   $r2 $r0 $r1              ; function selector comparison
jnzi $r2 i15                  ; jump to selected function
lw   $r1 data_4               ; load fn selector for comparison
eq   $r2 $r0 $r1              ; function selector comparison
jnzi $r2 i40                  ; jump to selected function
movi $$tmp i123               ; special code for mismatched selector
rvrt $$tmp                    ; revert if no selectors matched
lw   $r3 $fp i74              ; base register for method parameter
move $r2 $sp                  ; save locals base register
cfei i96                      ; allocate 96 bytes for locals
addi $r0 $r2 i0               ; get offset reg for get_ptr
lw   $r1 data_0               ; literal instantiation
addi $r0 $r2 i0               ; get store offset
mcpi $r0 $r1 i32              ; store value
addi $r0 $r2 i32              ; get offset reg for get_ptr
addi $r0 $r2 i32              ; get store offset
mcpi $r0 $r3 i64              ; store value
addi $r0 $r2 i32              ; get offset reg for get_ptr
addi $r0 $r2 i32              ; get offset
addi $r1 $r2 i0               ; get offset
swwq $r1 $r0                  ; quad word state access
addi $r0 $r2 i0               ; get offset reg for get_ptr
lw   $r1 data_1               ; literal instantiation
addi $r0 $r2 i0               ; get store offset
mcpi $r0 $r1 i32              ; store value
addi $r0 $r2 i64              ; get offset reg for get_ptr
addi $r1 $r2 i64              ; get offset
addi $r0 $r2 i0               ; get offset
swwq $r0 $r1                  ; quad word state access
ret  $zero                    ; returning unit as zero
move $r3 $sp                  ; save locals base register
cfei i96                      ; allocate 96 bytes for locals
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
.data:
data_0 .bytes[32] f3 83 b0 ce 51 35 8b e5 7d aa 3b 72 5f e4 4a cd b2 d8 80 60 4e 36 71 99 08 0b 43 79 c4 1b b6 ed  ....Q5..}.;r_.J....`N6q...Cy....
data_1 .bytes[32] f3 83 b0 ce 51 35 8b e5 7d aa 3b 72 5f e4 4a cd b2 d8 80 60 4e 36 71 99 08 0b 43 79 c4 1b b6 ee  ....Q5..}.;r_.J....`N6q...Cy....
data_2 .word 40
data_3 .word 3862599475
data_4 .word 3099753913
