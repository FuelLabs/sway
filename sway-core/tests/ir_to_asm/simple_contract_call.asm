.program:
ji   i4
noop
DATA_SECTION_OFFSET[0..32]
DATA_SECTION_OFFSET[32..64]
lw   $ds $is 1
add  $$ds $$ds $is
move $r4 $sp                  ; save locals base register
cfei i160                     ; allocate 160 bytes for all locals
addi $r1 $r4 i80              ; get_ptr
lw   $r0 data_0               ; literal instantiation
sw   $r1 $r0 i0               ; insert_value @ 0
move $r2 $sp                  ; save register for temporary stack value
cfei i48                      ; allocate 48 bytes for temporary struct
lw   $r1 data_1               ; literal instantiation
addi $r0 $r2 i0               ; get struct field(s) 0 offset
mcpi $r0 $r1 i32              ; store struct field value
lw   $r0 data_2               ; literal instantiation
sw   $r2 $r0 i4               ; insert_value @ 1
addi $r0 $r4 i80              ; get_ptr
sw   $r2 $r0 i5               ; insert_value @ 2
lw   $r1 data_3               ; literal instantiation
lw   $r0 data_4               ; literal instantiation
lw   $r3 data_5               ; literal instantiation
call $r2 $r1 $r0 $r3          ; call external contract
move $r1 $ret
addi $r0 $r4 i0               ; get_ptr
sw   $r4 $r1 i0               ; store value
addi $r0 $r4 i8               ; get_ptr
lw   $r1 data_6               ; literal instantiation
addi $r0 $r0 i0               ; get struct field(s) 0 offset
mcpi $r0 $r1 i32              ; store struct field value
move $r3 $sp                  ; save register for temporary stack value
cfei i48                      ; allocate 48 bytes for temporary struct
lw   $r1 data_1               ; literal instantiation
addi $r0 $r3 i0               ; get struct field(s) 0 offset
mcpi $r0 $r1 i32              ; store struct field value
lw   $r0 data_7               ; literal instantiation
sw   $r3 $r0 i4               ; insert_value @ 1
addi $r0 $r4 i8               ; get_ptr
sw   $r3 $r0 i5               ; insert_value @ 2
lw   $r2 data_3               ; literal instantiation
lw   $r1 data_4               ; literal instantiation
lw   $r0 data_8               ; literal instantiation
call $r3 $r2 $r1 $r0          ; call external contract
move $r1 $ret
addi $r0 $r4 i88              ; get_ptr
addi $r0 $r4 i88              ; get store offset
mcpi $r0 $r1 i32              ; store value
addi $r2 $r4 i40              ; get_ptr
lw   $r0 data_9               ; literal instantiation
sw   $r2 $r0 i0               ; insert_value @ 0
lw   $r1 data_10              ; literal instantiation
addi $r0 $r2 i8               ; get struct field(s) 1 offset
mcpi $r0 $r1 i32              ; store struct field value
move $r3 $sp                  ; save register for temporary stack value
cfei i48                      ; allocate 48 bytes for temporary struct
lw   $r1 data_1               ; literal instantiation
addi $r0 $r3 i0               ; get struct field(s) 0 offset
mcpi $r0 $r1 i32              ; store struct field value
lw   $r0 data_11              ; literal instantiation
sw   $r3 $r0 i4               ; insert_value @ 1
addi $r0 $r4 i40              ; get_ptr
sw   $r3 $r0 i5               ; insert_value @ 2
move $r2 $cgas                ; move register into abi function
lw   $r1 data_3               ; literal instantiation
lw   $r0 data_4               ; literal instantiation
call $r3 $r1 $r0 $r2          ; call external contract
move $r1 $ret
addi $r0 $r4 i120             ; get_ptr
addi $r0 $r4 i120             ; get store offset
mcpi $r0 $r1 i40              ; store value
lw   $r0 data_3               ; literal instantiation
ret  $r0
.data:
data_0 .u64 0x457
data_1 .b256 0x0c1c50c2bf5ba4bb351b4249a2f5e7d86556fcb4a6ae90465ff6c86126eeb3c0
data_2 .u64 0x9890aef4
data_3 .u64 0x00
data_4 .b256 0x0000000000000000000000000000000000000000000000000000000000000000
data_5 .u64 0x2710
data_6 .b256 0x3333333333333333333333333333333333333333333333333333333333333333
data_7 .u64 0x42123b96
data_8 .u64 0x4e20
data_9 .u64 0x15b3
data_10 .b256 0x5555555555555555555555555555555555555555555555555555555555555555
data_11 .u64 0xfc62d029
