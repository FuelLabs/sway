.program:
ji   i4
noop
DATA_SECTION_OFFSET[0..32]
DATA_SECTION_OFFSET[32..64]
lw   $ds $is 1
add  $$ds $$ds $is
move $r3 $sp                  ; save locals base register
cfei i160                     ; allocate 160 bytes for locals
addi $r1 $r3 i80              ; get offset reg for get_ptr
lw   $r0 data_0               ; literal instantiation
sw   $r1 $r0 i0               ; insert_value @ 0
lw   $r2 data_1               ; literal instantiation
lw   $r1 data_2               ; literal instantiation
addi $r0 $r2 i0               ; get struct field(s) 0 offset
mcpi $r0 $r1 i32              ; store struct field value
lw   $r0 data_3               ; literal instantiation
sw   $r2 $r0 i4               ; insert_value @ 1
addi $r0 $r3 i80              ; get offset reg for get_ptr
sw   $r2 $r0 i5               ; insert_value @ 2
lw   $r0 data_4               ; literal instantiation
lw   $r1 data_5               ; literal instantiation
call $r2 $zero $r0 $r1        ; call external contract
move $r1 $ret                 ; save call result
addi $r0 $r3 i0               ; get offset reg for get_ptr
sw   $r3 $r1 i0               ; store value
addi $r0 $r3 i8               ; get offset reg for get_ptr
lw   $r1 data_6               ; literal instantiation
addi $r0 $r0 i0               ; get struct field(s) 0 offset
mcpi $r0 $r1 i32              ; store struct field value
lw   $r2 data_1               ; literal instantiation
lw   $r1 data_2               ; literal instantiation
addi $r0 $r2 i0               ; get struct field(s) 0 offset
mcpi $r0 $r1 i32              ; store struct field value
lw   $r0 data_7               ; literal instantiation
sw   $r2 $r0 i4               ; insert_value @ 1
addi $r0 $r3 i8               ; get offset reg for get_ptr
sw   $r2 $r0 i5               ; insert_value @ 2
lw   $r1 data_4               ; literal instantiation
lw   $r0 data_8               ; literal instantiation
call $r2 $zero $r1 $r0        ; call external contract
move $r1 $ret                 ; save call result
addi $r0 $r3 i88              ; get offset reg for get_ptr
addi $r0 $r3 i88              ; get store offset
mcpi $r0 $r1 i32              ; store value
addi $r2 $r3 i40              ; get offset reg for get_ptr
lw   $r0 data_9               ; literal instantiation
sw   $r2 $r0 i0               ; insert_value @ 0
lw   $r1 data_10              ; literal instantiation
addi $r0 $r2 i8               ; get struct field(s) 1 offset
mcpi $r0 $r1 i32              ; store struct field value
lw   $r2 data_1               ; literal instantiation
lw   $r1 data_2               ; literal instantiation
addi $r0 $r2 i0               ; get struct field(s) 0 offset
mcpi $r0 $r1 i32              ; store struct field value
lw   $r0 data_11              ; literal instantiation
sw   $r2 $r0 i4               ; insert_value @ 1
addi $r0 $r3 i40              ; get offset reg for get_ptr
sw   $r2 $r0 i5               ; insert_value @ 2
move $r1 $cgas                ; move register into abi function
lw   $r0 data_4               ; literal instantiation
call $r2 $zero $r0 $r1        ; call external contract
move $r1 $ret                 ; save call result
addi $r0 $r3 i120             ; get offset reg for get_ptr
addi $r0 $r3 i120             ; get store offset
mcpi $r0 $r1 i40              ; store value
ret  $zero
noop                          ; word-alignment of data section
.data:
data_0 .word 1111
data_1 .collection { .word 0, .word 0, .word 0 }
data_2 .bytes[32] 0c 1c 50 c2 bf 5b a4 bb 35 1b 42 49 a2 f5 e7 d8 65 56 fc b4 a6 ae 90 46 5f f6 c8 61 26 ee b3 c0  ..P..[..5.BI....eV.....F_..a&...
data_3 .word 2559618804
data_4 .bytes[32] 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00  ................................
data_5 .word 10000
data_6 .bytes[32] 33 33 33 33 33 33 33 33 33 33 33 33 33 33 33 33 33 33 33 33 33 33 33 33 33 33 33 33 33 33 33 33  33333333333333333333333333333333
data_7 .word 1108491158
data_8 .word 20000
data_9 .word 5555
data_10 .bytes[32] 55 55 55 55 55 55 55 55 55 55 55 55 55 55 55 55 55 55 55 55 55 55 55 55 55 55 55 55 55 55 55 55  UUUUUUUUUUUUUUUUUUUUUUUUUUUUUUUU
data_11 .word 4234334249
