.program:
ji   i4
noop
DATA_SECTION_OFFSET[0..32]
DATA_SECTION_OFFSET[32..64]
lw   $ds $is 1
add  $$ds $$ds $is
lw   $r0 $fp i73              ; load input function selector
lw   $r1 data_5               ; load fn selector for comparison
eq   $r2 $r0 $r1              ; function selector comparison
jnzi $r2 i15                  ; jump to selected function
lw   $r1 data_6               ; load fn selector for comparison
eq   $r2 $r0 $r1              ; function selector comparison
jnzi $r2 i23                  ; jump to selected function
movi $$tmp i123               ; special code for mismatched selector
rvrt $$tmp                    ; revert if no selectors matched
lw   $r2 data_0               ; literal instantiation
lw   $r1 data_1               ; literal instantiation
addi $r0 $r2 i0               ; get struct field(s) 0 offset
mcpi $r0 $r1 i8               ; store struct field value
lw   $r0 data_2               ; loading size for RETD
retd  $r2 $r0
lw   $r1 data_0               ; literal instantiation
lw   $r0 data_3               ; literal instantiation
addi $r2 $r1 i0               ; get struct field(s) 0 offset
mcpi $r2 $r0 i16              ; store struct field value
lw   $r0 data_4               ; loading size for RETD
retd  $r1 $r0
.data:
data_0 .collection { .word 0 }
data_1 .bytes[7] 66 6f 6f 62 61 72 30  foobar0
data_2 .word 8
data_3 .bytes[9] 66 6f 6f 62 61 72 62 61 7a  foobarbaz
data_4 .word 16
data_5 .word 1242807808
data_6 .word 703232372
