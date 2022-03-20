.program:
ji   i4
noop
DATA_SECTION_OFFSET[0..32]
DATA_SECTION_OFFSET[32..64]
lw   $ds $is 1
add  $$ds $$ds $is
move $r7 $sp                  ; save locals base register
cfei i72                      ; allocate 72 bytes for all locals
lw   $r6 data_0               ; literal instantiation
lw   $r5 data_1               ; literal instantiation
lw   $r4 data_2               ; literal instantiation
lw   $r3 data_3               ; literal instantiation
move $r2 $sp                  ; stack pointer for args bundle
cfei i8
lw   $r0 data_4               ; literal instantiation
sw   $r2 $r0 i0               ; Get arg get_u64
lw   $r1 data_5               ; load fn selector for call
move $r0 $sp                  ; save register for temporary stack value
cfei i48
mcpi $r0 $r6 i32              ; copy contract address for call
sw   $r0 $r1 i4               ; write fn selector to rA + 32 for call
sw   $r0 $r2 i5               ; move user param for call
call $r0 $r5 $r4 $r3          ; call external contract
move $r0 $ret
lw   $r6 data_0               ; literal instantiation
lw   $r5 data_1               ; literal instantiation
lw   $r4 data_2               ; literal instantiation
lw   $r3 data_6               ; literal instantiation
move $r2 $sp                  ; stack pointer for args bundle
cfei i32
lw   $r1 data_7               ; literal instantiation
addi $r0 $r2 i0               ; get arg offset
mcpi $r0 $r1 i32              ; store arg field value
lw   $r1 data_8               ; load fn selector for call
move $r0 $sp                  ; save register for temporary stack value
cfei i48
mcpi $r0 $r6 i32              ; copy contract address for call
sw   $r0 $r1 i4               ; write fn selector to rA + 32 for call
sw   $r0 $r2 i5               ; move user param for call
call $r0 $r5 $r4 $r3          ; call external contract
move $r1 $ret
addi $r0 $r7 i0               ; get_ptr
addi $r0 $r7 i0               ; get store offset
mcpi $r0 $r1 i32              ; store value
lw   $r6 data_0               ; literal instantiation
lw   $r5 data_1               ; literal instantiation
lw   $r4 data_2               ; literal instantiation
lw   $r3 $cgas i0             ; loading $cgas (gas) into abi function
move $r2 $sp                  ; stack pointer for args bundle
cfei i40
lw   $r0 data_9               ; literal instantiation
sw   $r2 $r0 i0               ; Get arg get_s
lw   $r1 data_10              ; literal instantiation
addi $r0 $r2 i8               ; get arg offset
mcpi $r0 $r1 i32              ; store arg field value
lw   $r1 data_11              ; load fn selector for call
move $r0 $sp                  ; save register for temporary stack value
cfei i48
mcpi $r0 $r6 i32              ; copy contract address for call
sw   $r0 $r1 i4               ; write fn selector to rA + 32 for call
sw   $r0 $r2 i5               ; move user param for call
call $r0 $r5 $r4 $r3          ; call external contract
move $r1 $ret
addi $r0 $r7 i32              ; get_ptr
addi $r0 $r7 i32              ; get store offset
mcpi $r0 $r1 i40              ; store value
lw   $r0 data_1               ; literal instantiation
ret  $r0
noop                          ; word-alignment of data section
.data:
data_0 .b256 0x0c1c50c2bf5ba4bb351b4249a2f5e7d86556fcb4a6ae90465ff6c86126eeb3c0
data_1 .u64 0x00
data_2 .b256 0x0000000000000000000000000000000000000000000000000000000000000000
data_3 .u64 0x2710
data_4 .u64 0x457
data_5 .u32 0x9890aef4
data_6 .u64 0x4e20
data_7 .b256 0x3333333333333333333333333333333333333333333333333333333333333333
data_8 .u32 0x42123b96
data_9 .u64 0x15b3
data_10 .b256 0x5555555555555555555555555555555555555555555555555555555555555555
data_11 .u32 0xfc62d029
