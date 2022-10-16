.program:
ji   i4
noop
DATA_SECTION_OFFSET[0..32]
DATA_SECTION_OFFSET[32..64]
lw   $ds $is 1
add  $$ds $$ds $is
lw   $r0 $fp i73              ; load input function selector
lw   $r1 data_9               ; load fn selector for comparison
eq   $r2 $r0 $r1              ; function selector comparison
jnzi $r2 i12                  ; jump to selected function
movi $$tmp i123               ; special code for mismatched selector
rvrt $$tmp                    ; revert if no selectors matched
move $r5 $sp                  ; save locals base register
cfei i256                     ; allocate 256 bytes for locals
addi $r0 $r5 i0               ; get offset reg for get_ptr
lw   $r1 data_0               ; literal instantiation
addi $r0 $r5 i0               ; get store offset
mcpi $r0 $r1 i32              ; store value
addi $r0 $r5 i0               ; get offset
srw  $r0 $r0                  ; single word state access
lw   $r1 data_1               ; literal instantiation
sw   $r1 $r0 i0               ; insert_value @ 0
addi $r0 $r5 i32              ; get offset reg for get_ptr
lw   $r0 data_2               ; literal instantiation
addi $r2 $r5 i32              ; get store offset
mcpi $r2 $r0 i32              ; store value
addi $r3 $r5 i128             ; get offset reg for get_ptr
addi $r0 $r5 i128             ; get offset reg for get_ptr
addi $r2 $r5 i128             ; get offset
addi $r0 $r5 i32              ; get offset
srwq $r2 $r0                  ; quad word state access
addi $r0 $r5 i32              ; get offset reg for get_ptr
lw   $r2 data_3               ; literal instantiation
addi $r0 $r5 i32              ; get store offset
mcpi $r0 $r2 i32              ; store value
addi $r0 $r5 i160             ; get offset reg for get_ptr
addi $r2 $r5 i160             ; get offset
addi $r0 $r5 i32              ; get offset
srwq $r2 $r0                  ; quad word state access
addi $r0 $r1 i8               ; get struct field(s) 1 offset
mcpi $r0 $r3 i40              ; store struct field value
addi $r0 $r5 i64              ; get offset reg for get_ptr
lw   $r2 data_4               ; literal instantiation
addi $r0 $r5 i64              ; get store offset
mcpi $r0 $r2 i32              ; store value
addi $r0 $r5 i64              ; get offset
srw  $r0 $r0                  ; single word state access
lw   $r4 data_1               ; literal instantiation
sw   $r4 $r0 i0               ; insert_value @ 0
addi $r0 $r5 i96              ; get offset reg for get_ptr
lw   $r2 data_5               ; literal instantiation
addi $r0 $r5 i96              ; get store offset
mcpi $r0 $r2 i32              ; store value
addi $r3 $r5 i192             ; get offset reg for get_ptr
addi $r0 $r5 i192             ; get offset reg for get_ptr
addi $r2 $r5 i192             ; get offset
addi $r0 $r5 i96              ; get offset
srwq $r2 $r0                  ; quad word state access
addi $r0 $r5 i96              ; get offset reg for get_ptr
lw   $r2 data_6               ; literal instantiation
addi $r0 $r5 i96              ; get store offset
mcpi $r0 $r2 i32              ; store value
addi $r0 $r5 i224             ; get offset reg for get_ptr
addi $r2 $r5 i224             ; get offset
addi $r0 $r5 i96              ; get offset
srwq $r2 $r0                  ; quad word state access
addi $r0 $r4 i8               ; get struct field(s) 1 offset
mcpi $r0 $r3 i40              ; store struct field value
lw   $r2 data_7               ; literal instantiation
addi $r0 $r2 i0               ; get struct field(s) 0 offset
mcpi $r0 $r1 i48              ; store struct field value
addi $r0 $r2 i48              ; get struct field(s) 1 offset
mcpi $r0 $r4 i48              ; store struct field value
lw   $r0 data_8               ; loading size for RETD
retd  $r2 $r0
.data:
data_0 .bytes[32] d6 25 ff 6d 8e 88 ef d7 bb 34 76 e7 48 e5 d5 93 56 18 d7 8b fc 7e ed f5 84 fe 90 9c e0 80 9f c3  .%.m.....4v.H...V....~..........
data_1 .collection { .word 0, .word 0 }
data_2 .bytes[32] c4 f2 9c ca 5a 72 66 ec bc 35 c8 2c 55 dd 2b 00 59 a3 db 4c 83 a3 41 06 53 ec 33 ad ed 8e 98 40  ....Zrf..5.,U.+.Y..L..A.S.3....@
data_3 .bytes[32] c4 f2 9c ca 5a 72 66 ec bc 35 c8 2c 55 dd 2b 00 59 a3 db 4c 83 a3 41 06 53 ec 33 ad ed 8e 98 41  ....Zrf..5.,U.+.Y..L..A.S.3....A
data_4 .bytes[32] 28 17 e0 81 9d 6f ca d7 97 11 4f bc f3 50 fa 28 1a ca 33 a3 9b 0a bf 97 77 97 bd dd 69 b8 e7 af  (....o....O..P.(..3.....w...i...
data_5 .bytes[32] 12 ea 9b 9b 05 21 4a 0d 64 99 6d 25 9c 59 20 2b 80 a2 14 15 bb 68 b8 31 21 35 3e 2a 59 25 ec 47  .....!J.d.m%.Y +.....h.1!5>*Y%.G
data_6 .bytes[32] 12 ea 9b 9b 05 21 4a 0d 64 99 6d 25 9c 59 20 2b 80 a2 14 15 bb 68 b8 31 21 35 3e 2a 59 25 ec 48  .....!J.d.m%.Y +.....h.1!5>*Y%.H
data_7 .collection { .collection { .word 0, .word 0 }, .collection { .word 0, .word 0 } }
data_8 .word 96
data_9 .word 23485428
