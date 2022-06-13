.program:
ji   i4
noop
DATA_SECTION_OFFSET[0..32]
DATA_SECTION_OFFSET[32..64]
lw   $ds $is 1
add  $$ds $$ds $is
lw   $r1 $fp i73              ; load input function selector
lw   $r0 data_8               ; load fn selector for comparison
eq   $r0 $r1 $r0              ; function selector comparison
jnzi $r0 i11                  ; jump to selected function
rvrt $zero                    ; revert if no selectors matched
move $r5 $sp                  ; save locals base register
cfei i256                     ; allocate 256 bytes for all locals
lw   $r0 $fp i74              ; Base register for method parameter
addi $r1 $r0 i0               ; Get address for arg s
lw   $r3 $r0 i5               ; Get arg u
move $r2 $sp                  ; save register for temporary stack value
cfei i48                      ; allocate 48 bytes for temporary struct
lw   $r0 data_0               ; literal instantiation for aggregate field
sw   $r2 $r0 i0               ; initialise aggregate field
addi $r0 $r2 i8               ; get struct field(s) 1 offset
mcpi $r0 $r1 i40              ; store struct field value
lw   $r1 $r2 i0               ; extract_value @ 0
addi $r0 $r5 i0               ; get offset reg for get_ptr
lw   $r0 data_1               ; literal instantiation
addi $r4 $r5 i0               ; get store offset
mcpi $r4 $r0 i32              ; store value
addi $r0 $r5 i0               ; get offset
sww  $r0 $r1                  ; single word state access
addi $r2 $r2 i8               ; extract address
addi $r0 $r5 i32              ; get offset reg for get_ptr
lw   $r1 data_2               ; literal instantiation
addi $r0 $r5 i32              ; get store offset
mcpi $r0 $r1 i32              ; store value
addi $r0 $r5 i128             ; get offset reg for get_ptr
addi $r0 $r5 i128             ; get store offset
mcpi $r0 $r2 i64              ; store value
addi $r0 $r5 i128             ; get offset reg for get_ptr
addi $r1 $r5 i128             ; get offset
addi $r0 $r5 i32              ; get offset
swwq $r0 $r1                  ; quad word state access
addi $r0 $r5 i32              ; get offset reg for get_ptr
lw   $r1 data_3               ; literal instantiation
addi $r0 $r5 i32              ; get store offset
mcpi $r0 $r1 i32              ; store value
addi $r0 $r5 i160             ; get offset reg for get_ptr
addi $r1 $r5 i160             ; get offset
addi $r0 $r5 i32              ; get offset
swwq $r0 $r1                  ; quad word state access
move $r4 $sp                  ; save register for temporary stack value
cfei i48                      ; allocate 48 bytes for temporary struct
lw   $r0 data_4               ; literal instantiation for aggregate field
sw   $r4 $r0 i0               ; initialise aggregate field
sw   $r4 $r3 i5               ; insert_value @ 1
lw   $r2 $r4 i0               ; extract_value @ 0
addi $r0 $r5 i64              ; get offset reg for get_ptr
lw   $r1 data_5               ; literal instantiation
addi $r0 $r5 i64              ; get store offset
mcpi $r0 $r1 i32              ; store value
addi $r0 $r5 i64              ; get offset
sww  $r0 $r2                  ; single word state access
addi $r2 $r4 i8               ; extract address
addi $r0 $r5 i96              ; get offset reg for get_ptr
lw   $r1 data_6               ; literal instantiation
addi $r0 $r5 i96              ; get store offset
mcpi $r0 $r1 i32              ; store value
addi $r0 $r5 i192             ; get offset reg for get_ptr
addi $r0 $r5 i192             ; get store offset
mcpi $r0 $r2 i64              ; store value
addi $r0 $r5 i192             ; get offset reg for get_ptr
addi $r1 $r5 i192             ; get offset
addi $r0 $r5 i96              ; get offset
swwq $r0 $r1                  ; quad word state access
addi $r0 $r5 i96              ; get offset reg for get_ptr
lw   $r1 data_7               ; literal instantiation
addi $r0 $r5 i96              ; get store offset
mcpi $r0 $r1 i32              ; store value
addi $r0 $r5 i224             ; get offset reg for get_ptr
addi $r1 $r5 i224             ; get offset
addi $r0 $r5 i96              ; get offset
swwq $r0 $r1                  ; quad word state access
ret  $zero                    ; returning unit as zero
noop                          ; word-alignment of data section
.data:
data_0 .u64 0x00
data_1 .b256 0xd625ff6d8e88efd7bb3476e748e5d5935618d78bfc7eedf584fe909ce0809fc3
data_2 .b256 0xc4f29cca5a7266ecbc35c82c55dd2b0059a3db4c83a3410653ec33aded8e9840
data_3 .b256 0xc4f29cca5a7266ecbc35c82c55dd2b0059a3db4c83a3410653ec33aded8e9841
data_4 .u64 0x01
data_5 .b256 0x2817e0819d6fcad797114fbcf350fa281aca33a39b0abf977797bddd69b8e7af
data_6 .b256 0x12ea9b9b05214a0d64996d259c59202b80a21415bb68b83121353e2a5925ec47
data_7 .b256 0x12ea9b9b05214a0d64996d259c59202b80a21415bb68b83121353e2a5925ec48
data_8 .u32 0xc1c7877c
