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
jnzi $r2 i29                  ; jump to selected function
movi $$tmp i123               ; special code for mismatched selector
rvrt $$tmp                    ; revert if no selectors matched
move $r2 $sp                  ; save locals base register
cfei i40                      ; allocate 40 bytes for locals
addi $r0 $r2 i0               ; get offset reg for get_ptr
lw   $r1 data_0               ; literal instantiation
addi $r0 $r2 i0               ; get store offset
mcpi $r0 $r1 i32              ; store value
addi $r0 $r2 i32              ; get offset reg for get_ptr
addi $r0 $r2 i0               ; get offset
srw  $r0 $r0                  ; single word state access
sw   $r2 $r0 i4               ; store value
addi $r0 $r2 i32              ; get offset reg for get_ptr
lw   $r0 $r2 i4               ; load value
ret  $r0
move $r2 $sp                  ; save locals base register
cfei i64                      ; allocate 64 bytes for locals
addi $r0 $r2 i0               ; get offset reg for get_ptr
lw   $r1 data_1               ; literal instantiation
addi $r0 $r2 i0               ; get store offset
mcpi $r0 $r1 i32              ; store value
addi $r0 $r2 i32              ; get offset reg for get_ptr
addi $r1 $r2 i32              ; get offset
addi $r0 $r2 i0               ; get offset
srwq $r1 $r0                  ; quad word state access
addi $r0 $r2 i32              ; get offset reg for get_ptr
addi $r1 $r2 i32              ; load address
lw   $r0 data_2               ; loading size for RETD
retd  $r1 $r0
noop                          ; word-alignment of data section
.data:
data_0 .bytes[32] 7f bd 11 92 66 6b fa c3 76 7b 89 0b d4 d0 48 c9 40 87 9d 31 60 71 e2 0c 7c 8c 81 bc e2 ca 41 c5  ....fk..v{....H.@..1`q..|.....A.
data_1 .bytes[32] a1 5d 6d 36 b5 4d f9 93 ed 1f be 45 44 a4 5d 4c 4f 70 d8 1b 42 29 86 1d fd e0 e2 0e b6 52 20 2c  .]m6.M.....ED.]LOp..B).......R ,
data_2 .word 32
data_3 .word 2384949349
data_4 .word 1151241875
