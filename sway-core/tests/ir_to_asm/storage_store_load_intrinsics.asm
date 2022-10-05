.program:
ji   i4
noop
DATA_SECTION_OFFSET[0..32]
DATA_SECTION_OFFSET[32..64]
lw   $ds $is 1
add  $$ds $$ds $is
lw   $r1 $fp i73              ; load input function selector
lw   $r0 data_13              ; load fn selector for comparison
eq   $r0 $r1 $r0              ; function selector comparison
jnzi $r0 i11                  ; jump to selected function
rvrt $zero                    ; revert if no selectors matched
move $r4 $sp                  ; save locals base register
cfei i288                     ; allocate 288 bytes for all locals
lw   $r0 data_0               ; literal instantiation
addi $r1 $r4 i240             ; get offset reg for get_ptr
addi $r1 $r4 i240             ; get store offset
mcpi $r1 $r0 i32              ; store value
eq   $r0 $zero $zero          ; asm block
jnzi $r0 i21
ji   i31
addi $r0 $r4 i240             ; get offset reg for get_ptr
addi $r3 $r4 i240             ; load address
lw   $r1 data_1               ; literal instantiation
lw   $r0 data_2               ; literal instantiation
move $r2 $sp                  ; asm block
cfei i8                       ; asm block
sw   $r2 $r1 i0               ; asm block
s256 $r3 $r2 $r0              ; asm block
cfsi i8                       ; asm block
ji   i40
addi $r0 $r4 i272             ; get offset reg for get_ptr
lw   $r0 data_2               ; literal instantiation
sw   $r4 $r0 i34              ; store value
addi $r0 $r4 i240             ; get offset reg for get_ptr
addi $r3 $r4 i240             ; load address
addi $r0 $r4 i272             ; get offset reg for get_ptr
lw   $r1 $r4 i34              ; load value
lw   $r0 data_1               ; literal instantiation
s256 $r3 $r0 $r1              ; asm block
addi $r0 $r4 i0               ; get offset reg for get_ptr
addi $r0 $r4 i0               ; get store offset
mcpi $r0 $r3 i32              ; store value
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
jnzi $r0 i68
ji   i71
rvrt $zero                    ; asm block
move $r0 $zero                ; parameter from branch to block argument
ji   i72
move $r0 $zero                ; parameter from branch to block argument
move $r0 $zero                ; parameter from branch to block argument
addi $r0 $r4 i160             ; get offset reg for get_ptr
move $r1 $sp                  ; save register for temporary stack value
cfei i32                      ; allocate 32 bytes for temporary struct
lw   $r0 data_4               ; literal instantiation for aggregate field
sw   $r1 $r0 i0               ; initialise aggregate field
lw   $r0 data_5               ; literal instantiation for aggregate field
sw   $r1 $r0 i1               ; initialise aggregate field
lw   $r0 data_6               ; literal instantiation for aggregate field
sw   $r1 $r0 i2               ; initialise aggregate field
lw   $r0 data_7               ; literal instantiation for aggregate field
sw   $r1 $r0 i3               ; initialise aggregate field
addi $r0 $r4 i160             ; get store offset
mcpi $r0 $r1 i32              ; store value
addi $r0 $r4 i200             ; get offset reg for get_ptr
move $r1 $sp                  ; save register for temporary stack value
cfei i32                      ; allocate 32 bytes for temporary struct
lw   $r0 data_8               ; literal instantiation for aggregate field
sw   $r1 $r0 i0               ; initialise aggregate field
lw   $r0 data_9               ; literal instantiation for aggregate field
sw   $r1 $r0 i1               ; initialise aggregate field
lw   $r0 data_10              ; literal instantiation for aggregate field
sw   $r1 $r0 i2               ; initialise aggregate field
lw   $r0 data_11              ; literal instantiation for aggregate field
sw   $r1 $r0 i3               ; initialise aggregate field
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
jnzi $r0 i130
ji   i135
addi $r0 $r4 i160             ; get offset reg for get_ptr
lw   $r1 $r0 i1               ; extract_value @ 1
addi $r0 $r4 i200             ; get offset reg for get_ptr
lw   $r0 $r0 i1               ; extract_value @ 1
eq   $r0 $r1 $r0
jnzi $r0 i137
ji   i142
addi $r0 $r4 i160             ; get offset reg for get_ptr
lw   $r1 $r0 i2               ; extract_value @ 2
addi $r0 $r4 i200             ; get offset reg for get_ptr
lw   $r0 $r0 i2               ; extract_value @ 2
eq   $r0 $r1 $r0
jnzi $r0 i144
ji   i149
addi $r0 $r4 i160             ; get offset reg for get_ptr
lw   $r1 $r0 i3               ; extract_value @ 3
addi $r0 $r4 i200             ; get offset reg for get_ptr
lw   $r0 $r0 i3               ; extract_value @ 3
eq   $r0 $r1 $r0
eq   $r0 $r0 $zero            ; asm block
jnzi $r0 i152
ji   i155
rvrt $zero                    ; asm block
move $r0 $zero                ; parameter from branch to block argument
ji   i156
move $r0 $zero                ; parameter from branch to block argument
move $r0 $zero                ; parameter from branch to block argument
lw   $r0 data_12              ; literal instantiation
ret  $r0
noop                          ; word-alignment of data section
.data:
data_0 .b256 0x0000000000000000000000000000000000000000000000000000000000000000
data_1 .u64 0x16
data_2 .u64 0x08
data_3 .u64 0x6c
data_4 .u64 0x01
data_5 .u64 0x02
data_6 .u64 0x04
data_7 .u64 0x64
data_8 .u64 0x65
data_9 .u64 0x79
data_10 .u64 0xe0
data_11 .u64 0x68
data_12 .u64 0x80
data_13 .u32 0xea1a0f91
