.program:
ji   i4
noop
DATA_SECTION_OFFSET[0..32]
DATA_SECTION_OFFSET[32..64]
lw   $ds $is 1
add  $$ds $$ds $is
lw   $r0 $fp i73              ; load input function selector
lw   $r1 data_7               ; load fn selector for comparison
eq   $r2 $r0 $r1              ; function selector comparison
jnzi $r2 i12                  ; jump to selected function
movi $$tmp i123               ; special code for mismatched selector
rvrt $$tmp                    ; revert if no selectors matched
move $r4 $sp                  ; save locals base register
cfei i288                     ; allocate 288 bytes for locals
lw   $r0 data_0               ; literal instantiation
addi $r1 $r4 i240             ; get offset reg for get_ptr
addi $r1 $r4 i240             ; get store offset
mcpi $r1 $r0 i32              ; store value
eq   $r0 $zero $zero          ; asm block
jnzi $r0 i22
ji   i32
addi $r0 $r4 i240             ; get offset reg for get_ptr
addi $r2 $r4 i240             ; load address
lw   $r1 data_1               ; literal instantiation
lw   $r0 data_2               ; literal instantiation
move $r3 $sp                  ; asm block
cfei i8                       ; asm block
sw   $r3 $r1 i0               ; asm block
s256 $r2 $r3 $r0              ; asm block
cfsi i8                       ; asm block
ji   i41
addi $r0 $r4 i272             ; get offset reg for get_ptr
lw   $r0 data_2               ; literal instantiation
sw   $r4 $r0 i34              ; store value
addi $r0 $r4 i240             ; get offset reg for get_ptr
addi $r2 $r4 i240             ; load address
addi $r0 $r4 i272             ; get offset reg for get_ptr
lw   $r1 $r4 i34              ; load value
lw   $r0 data_1               ; literal instantiation
s256 $r2 $r0 $r1              ; asm block
addi $r0 $r4 i0               ; get offset reg for get_ptr
addi $r0 $r4 i0               ; get store offset
mcpi $r0 $r2 i32              ; store value
addi $r0 $r4 i280             ; get offset reg for get_ptr
lw   $r0 data_3               ; literal instantiation
sw   $r4 $r0 i35              ; store value
addi $r0 $r4 i0               ; get offset reg for get_ptr
addi $r2 $r4 i0               ; load address
addi $r0 $r4 i280             ; get offset reg for get_ptr
lw   $r1 $r4 i35              ; load value
addi $r0 $r4 i32              ; get offset reg for get_ptr
addi $r0 $r4 i32              ; get store offset
mcpi $r0 $r2 i32              ; store value
addi $r0 $r4 i32              ; get offset
sww  $r0 $r1                  ; single word state access
addi $r0 $r4 i0               ; get offset reg for get_ptr
addi $r1 $r4 i0               ; load address
addi $r0 $r4 i64              ; get offset reg for get_ptr
addi $r0 $r4 i64              ; get store offset
mcpi $r0 $r1 i32              ; store value
addi $r0 $r4 i64              ; get offset
srw  $r1 $r0                  ; single word state access
addi $r0 $r4 i280             ; get offset reg for get_ptr
lw   $r0 $r4 i35              ; load value
eq   $r0 $r1 $r0
eq   $r0 $r0 $zero            ; asm block
jnzi $r0 i69
ji   i71
rvrt $zero                    ; asm block
ji   i71
addi $r0 $r4 i160             ; get offset reg for get_ptr
lw   $r1 data_4               ; literal instantiation
addi $r0 $r4 i160             ; get store offset
mcpi $r0 $r1 i32              ; store value
addi $r0 $r4 i200             ; get offset reg for get_ptr
lw   $r1 data_5               ; literal instantiation
addi $r0 $r4 i200             ; get store offset
mcpi $r0 $r1 i32              ; store value
addi $r1 $r4 i160             ; get offset reg for get_ptr
addi $r0 $r4 i192             ; get offset reg for get_ptr
sw   $r4 $r1 i24              ; store value
addi $r1 $r4 i200             ; get offset reg for get_ptr
addi $r0 $r4 i232             ; get offset reg for get_ptr
sw   $r4 $r1 i29              ; store value
addi $r0 $r4 i0               ; get offset reg for get_ptr
addi $r2 $r4 i0               ; load address
addi $r0 $r4 i192             ; get offset reg for get_ptr
lw   $r1 $r4 i24              ; load value
addi $r0 $r4 i96              ; get offset reg for get_ptr
addi $r0 $r4 i96              ; get store offset
mcpi $r0 $r2 i32              ; store value
addi $r0 $r4 i96              ; get offset
swwq $r0 $r1                  ; quad word state access
addi $r0 $r4 i0               ; get offset reg for get_ptr
addi $r2 $r4 i0               ; load address
addi $r0 $r4 i232             ; get offset reg for get_ptr
lw   $r1 $r4 i29              ; load value
addi $r0 $r4 i128             ; get offset reg for get_ptr
addi $r0 $r4 i128             ; get store offset
mcpi $r0 $r2 i32              ; store value
addi $r0 $r4 i128             ; get offset
srwq $r1 $r0                  ; quad word state access
addi $r0 $r4 i160             ; get offset reg for get_ptr
lw   $r1 $r0 i0               ; extract_value @ 0
addi $r0 $r4 i200             ; get offset reg for get_ptr
lw   $r0 $r0 i0               ; extract_value @ 0
eq   $r0 $r1 $r0
jnzi $r0 i112
ji   i117
addi $r0 $r4 i160             ; get offset reg for get_ptr
lw   $r1 $r0 i1               ; extract_value @ 1
addi $r0 $r4 i200             ; get offset reg for get_ptr
lw   $r0 $r0 i1               ; extract_value @ 1
eq   $r0 $r1 $r0
jnzi $r0 i119
ji   i124
addi $r0 $r4 i160             ; get offset reg for get_ptr
lw   $r1 $r0 i2               ; extract_value @ 2
addi $r0 $r4 i200             ; get offset reg for get_ptr
lw   $r0 $r0 i2               ; extract_value @ 2
eq   $r0 $r1 $r0
jnzi $r0 i126
ji   i131
addi $r0 $r4 i160             ; get offset reg for get_ptr
lw   $r1 $r0 i3               ; extract_value @ 3
addi $r0 $r4 i200             ; get offset reg for get_ptr
lw   $r0 $r0 i3               ; extract_value @ 3
eq   $r0 $r1 $r0
eq   $r0 $r0 $zero            ; asm block
jnzi $r0 i134
ji   i136
rvrt $zero                    ; asm block
ji   i136
lw   $r0 data_6               ; literal instantiation
ret  $r0
move $$tmp $sp                ; save base stack value
cfei i16                      ; reserve space for saved registers
sw   $$tmp $r0 i0             ; save $r0
sw   $$tmp $r1 i1             ; save $r1
move $r0 $$arg0               ; save arg 0
move $r1 $$reta               ; save reta
eq   $r0 $r0 $zero            ; asm block
move $$retv $r0               ; set return value
move $$reta $r1               ; restore reta
subi $$tmp $sp i16            ; save base stack value
lw   $r0 $$tmp i0             ; restore $r0
lw   $r1 $$tmp i1             ; restore $r1
cfsi i16                      ; recover space from saved registers
jmp $$reta                    ; return from call
move $$tmp $sp                ; save base stack value
cfei i16                      ; reserve space for saved registers
sw   $$tmp $r0 i0             ; save $r0
sw   $$tmp $r1 i1             ; save $r1
move $r0 $$arg0               ; save arg 0
move $r1 $$reta               ; save reta
move $$arg0 $r0               ; pass arg 0
movi $$reta i161              ; set new return addr
ji   i175                     ; call not_4
move $r0 $$retv               ; copy the return value
jnzi $r0 i164
ji   i168
move $$arg0 $zero             ; pass arg 0
movi $$reta i167              ; set new return addr
ji   i189                     ; call revert_5
ji   i168
move $$retv $zero             ; set return value
move $$reta $r1               ; restore reta
subi $$tmp $sp i16            ; save base stack value
lw   $r0 $$tmp i0             ; restore $r0
lw   $r1 $$tmp i1             ; restore $r1
cfsi i16                      ; recover space from saved registers
jmp $$reta                    ; return from call
move $$tmp $sp                ; save base stack value
cfei i16                      ; reserve space for saved registers
sw   $$tmp $r0 i0             ; save $r0
sw   $$tmp $r1 i1             ; save $r1
move $r0 $$arg0               ; save arg 0
move $r1 $$reta               ; save reta
eq   $r0 $r0 $zero            ; asm block
move $$retv $r0               ; set return value
move $$reta $r1               ; restore reta
subi $$tmp $sp i16            ; save base stack value
lw   $r0 $$tmp i0             ; restore $r0
lw   $r1 $$tmp i1             ; restore $r1
cfsi i16                      ; recover space from saved registers
jmp $$reta                    ; return from call
move $$tmp $sp                ; save base stack value
cfei i16                      ; reserve space for saved registers
sw   $$tmp $r0 i0             ; save $r0
sw   $$tmp $r1 i1             ; save $r1
move $r1 $$arg0               ; save arg 0
move $r0 $$reta               ; save reta
rvrt $r1                      ; asm block
move $$retv $zero             ; set return value
move $$reta $r0               ; restore reta
subi $$tmp $sp i16            ; save base stack value
lw   $r0 $$tmp i0             ; restore $r0
lw   $r1 $$tmp i1             ; restore $r1
cfsi i16                      ; recover space from saved registers
jmp $$reta                    ; return from call
move $$tmp $sp                ; save base stack value
cfei i24                      ; reserve space for saved registers
sw   $$tmp $r0 i0             ; save $r0
sw   $$tmp $r1 i1             ; save $r1
sw   $$tmp $r2 i2             ; save $r2
move $r2 $$arg0               ; save arg 0
move $r0 $$arg1               ; save arg 1
move $r1 $$reta               ; save reta
eq   $r0 $r2 $r0
move $$retv $r0               ; set return value
move $$reta $r1               ; restore reta
subi $$tmp $sp i24            ; save base stack value
lw   $r0 $$tmp i0             ; restore $r0
lw   $r1 $$tmp i1             ; restore $r1
lw   $r2 $$tmp i2             ; restore $r2
cfsi i24                      ; recover space from saved registers
jmp $$reta                    ; return from call
move $$tmp $sp                ; save base stack value
cfei i24                      ; reserve space for saved registers
sw   $$tmp $r0 i0             ; save $r0
sw   $$tmp $r1 i1             ; save $r1
sw   $$tmp $r2 i2             ; save $r2
move $r2 $$arg0               ; save arg 0
move $r0 $$arg1               ; save arg 1
move $r1 $$reta               ; save reta
eq   $r0 $r2 $r0
move $$retv $r0               ; set return value
move $$reta $r1               ; restore reta
subi $$tmp $sp i24            ; save base stack value
lw   $r0 $$tmp i0             ; restore $r0
lw   $r1 $$tmp i1             ; restore $r1
lw   $r2 $$tmp i2             ; restore $r2
cfsi i24                      ; recover space from saved registers
jmp $$reta                    ; return from call
move $$tmp $sp                ; save base stack value
cfei i24                      ; reserve space for saved registers
sw   $$tmp $r0 i0             ; save $r0
sw   $$tmp $r1 i1             ; save $r1
sw   $$tmp $r2 i2             ; save $r2
move $r2 $$arg0               ; save arg 0
move $r0 $$arg1               ; save arg 1
move $r1 $$reta               ; save reta
eq   $r0 $r2 $r0
move $$retv $r0               ; set return value
move $$reta $r1               ; restore reta
subi $$tmp $sp i24            ; save base stack value
lw   $r0 $$tmp i0             ; restore $r0
lw   $r1 $$tmp i1             ; restore $r1
lw   $r2 $$tmp i2             ; restore $r2
cfsi i24                      ; recover space from saved registers
jmp $$reta                    ; return from call
move $$tmp $sp                ; save base stack value
cfei i24                      ; reserve space for saved registers
sw   $$tmp $r0 i0             ; save $r0
sw   $$tmp $r1 i1             ; save $r1
sw   $$tmp $r2 i2             ; save $r2
move $r2 $$arg0               ; save arg 0
move $r0 $$arg1               ; save arg 1
move $r1 $$reta               ; save reta
eq   $r0 $r2 $r0
move $$retv $r0               ; set return value
move $$reta $r1               ; restore reta
subi $$tmp $sp i24            ; save base stack value
lw   $r0 $$tmp i0             ; restore $r0
lw   $r1 $$tmp i1             ; restore $r1
lw   $r2 $$tmp i2             ; restore $r2
cfsi i24                      ; recover space from saved registers
jmp $$reta                    ; return from call
move $$tmp $sp                ; save base stack value
cfei i24                      ; reserve space for saved registers
sw   $$tmp $r0 i0             ; save $r0
sw   $$tmp $r1 i1             ; save $r1
sw   $$tmp $r2 i2             ; save $r2
move $r2 $$arg0               ; save arg 0
move $r0 $$arg1               ; save arg 1
move $r1 $$reta               ; save reta
eq   $r0 $r2 $r0
move $$retv $r0               ; set return value
move $$reta $r1               ; restore reta
subi $$tmp $sp i24            ; save base stack value
lw   $r0 $$tmp i0             ; restore $r0
lw   $r1 $$tmp i1             ; restore $r1
lw   $r2 $$tmp i2             ; restore $r2
cfsi i24                      ; recover space from saved registers
jmp $$reta                    ; return from call
.data:
data_0 .bytes[32] 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00  ................................
data_1 .word 22
data_2 .word 8
data_3 .word 108
data_4 .collection { .word 1, .word 2, .word 4, .word 100 }
data_5 .collection { .word 101, .word 121, .word 224, .word 104 }
data_6 .word 128
data_7 .word 3927576465
