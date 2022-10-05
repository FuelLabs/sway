.program:
ji   i4
noop
DATA_SECTION_OFFSET[0..32]
DATA_SECTION_OFFSET[32..64]
lw   $ds $is 1
add  $$ds $$ds $is
lw   $r0 $fp i73              ; load input function selector
lw   $r1 data_2               ; load fn selector for comparison
eq   $r2 $r0 $r1              ; function selector comparison
jnzi $r2 i15                  ; jump to selected function
lw   $r1 data_3               ; load fn selector for comparison
eq   $r2 $r0 $r1              ; function selector comparison
jnzi $r2 i18                  ; jump to selected function
movi $$tmp i123               ; special code for mismatched selector
rvrt $$tmp                    ; revert if no selectors matched
lw   $r1 $fp i74              ; base register for method parameter
lw   $r0 data_0               ; loading size for RETD
retd  $r1 $r0
lw   $r1 $fp i74              ; base register for method parameter
lw   $r0 data_1               ; loading size for RETD
retd  $r1 $r0
.data:
data_0 .word 8
data_1 .word 16
data_2 .word 2161799394
data_3 .word 683734681
