.program:
ji   i4
noop
DATA_SECTION_OFFSET[0..32]
DATA_SECTION_OFFSET[32..64]
lw   $ds $is 1
add  $$ds $$ds $is
lw   $r0 $fp i73              ; load input function selector
lw   $r1 data_2               ; load fn selector for comparison
eq   $r1 $r0 $r1              ; function selector comparison
jnei $zero $r1 i17            ; jump to selected function
lw   $r1 data_3               ; load fn selector for comparison
eq   $r1 $r0 $r1              ; function selector comparison
jnei $zero $r1 i19            ; jump to selected function
lw   $r1 data_4               ; load fn selector for comparison
eq   $r0 $r0 $r1              ; function selector comparison
jnei $zero $r0 i22            ; jump to selected function
rvrt $zero                    ; revert if no selectors matched
lw   $r0 $fp i74              ; Base register for method parameter
ret  $r0
lw   $r1 $fp i74              ; Base register for method parameter
lw   $r0 data_0               ; loading size for RETD
retd  $r1 $r0
lw   $r1 $fp i74              ; Base register for method parameter
lw   $r0 $r1 i0               ; Get arg val1
addi $r2 $r1 i8               ; Get address for arg val2
move $r1 $sp                  ; save register for temporary stack value
cfei i40                      ; allocate 40 bytes for temporary struct
sw   $r1 $r0 i0               ; insert_value @ 0
addi $r0 $r1 i8               ; get struct field(s) 1 offset
mcpi $r0 $r2 i32              ; store struct field value
lw   $r0 data_1               ; loading size for RETD
retd  $r1 $r0
noop                          ; word-alignment of data section
.data:
data_0 .u64 0x20
data_1 .u64 0x28
data_2 .u32 0x9890aef4
data_3 .u32 0x42123b96
data_4 .u32 0xfc62d029
