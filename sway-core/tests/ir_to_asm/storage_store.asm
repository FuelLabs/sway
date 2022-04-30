.program:
ji   i4
noop
DATA_SECTION_OFFSET[0..32]
DATA_SECTION_OFFSET[32..64]
lw   $ds $is 1
add  $$ds $$ds $is
lw   $r1 $fp i73              ; load input function selector
lw   $r0 data_4               ; load fn selector for comparison
eq   $r0 $r1 $r0              ; function selector comparison
jnzi $r0 i14                  ; jump to selected function
lw   $r0 data_5               ; load fn selector for comparison
eq   $r0 $r1 $r0              ; function selector comparison
jnzi $r0 i28                  ; jump to selected function
rvrt $zero                    ; revert if no selectors matched
move $r2 $sp                  ; save locals base register
cfei i40                      ; allocate 40 bytes for all locals
addi $r0 $r2 i32              ; get_ptr
lw   $r0 data_0               ; literal instantiation
sw   $r2 $r0 i4               ; store value
addi $r0 $r2 i0               ; get_ptr
lw   $r1 data_1               ; literal instantiation
addi $r0 $r2 i0               ; get store offset
mcpi $r0 $r1 i32              ; store value
lw   $r1 data_0               ; literal instantiation
addi $r0 $r2 i0               ; get offset
sww  $r0 $r1                  ; single word state access
ret  $zero                    ; returning unit as zero
move $r1 $sp                  ; save locals base register
cfei i64                      ; allocate 64 bytes for all locals
addi $r0 $r1 i32              ; get_ptr
lw   $r2 data_2               ; literal instantiation
addi $r0 $r1 i32              ; get store offset
mcpi $r0 $r2 i32              ; store value
addi $r0 $r1 i0               ; get_ptr
lw   $r2 data_3               ; literal instantiation
addi $r0 $r1 i0               ; get store offset
mcpi $r0 $r2 i32              ; store value
addi $r0 $r1 i32              ; get_ptr
addi $r2 $r1 i32              ; get offset
addi $r0 $r1 i0               ; get offset
swwq $r0 $r2                  ; quad word state access
ret  $zero                    ; returning unit as zero
noop                          ; word-alignment of data section
.data:
data_0 .u64 0x00
data_1 .b256 0x7fbd1192666bfac3767b890bd4d048c940879d316071e20c7c8c81bce2ca41c5
data_2 .b256 0x0000000000000000000000000000000000000000000000000000000000000000
data_3 .b256 0xa15d6d36b54df993ed1fbe4544a45d4c4f70d81b4229861dfde0e20eb652202c
data_4 .u32 0x1b9b478f
data_5 .u32 0x858a3d18
