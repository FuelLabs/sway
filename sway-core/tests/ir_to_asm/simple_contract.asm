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
jnzi $r2 i18                  ; jump to selected function
lw   $r1 data_4               ; load fn selector for comparison
eq   $r2 $r0 $r1              ; function selector comparison
jnzi $r2 i20                  ; jump to selected function
lw   $r1 data_5               ; load fn selector for comparison
eq   $r2 $r0 $r1              ; function selector comparison
jnzi $r2 i23                  ; jump to selected function
movi $$tmp i123               ; special code for mismatched selector
rvrt $$tmp                    ; revert if no selectors matched
lw   $r0 $fp i74              ; base register for method parameter
ret  $r0
lw   $r1 $fp i74              ; base register for method parameter
lw   $r0 data_0               ; loading size for RETD
retd  $r1 $r0
lw   $r0 $fp i74              ; base register for method parameter
lw   $r1 $r0 i0               ; get arg val1
addi $r0 $r0 i8               ; get address for arg val2
lw   $r2 data_1               ; literal instantiation
sw   $r2 $r1 i0               ; insert_value @ 0
addi $r1 $r2 i8               ; get struct field(s) 1 offset
mcpi $r1 $r0 i32              ; store struct field value
lw   $r0 data_2               ; loading size for RETD
retd  $r2 $r0
noop                          ; word-alignment of data section
.data:
data_0 .word 32
data_1 .collection { .word 0, .word 0 }
data_2 .word 40
data_3 .word 2559618804
data_4 .word 1108491158
data_5 .word 4234334249
