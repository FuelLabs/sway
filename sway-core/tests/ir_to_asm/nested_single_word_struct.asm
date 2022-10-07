.program:
ji   i4
noop
DATA_SECTION_OFFSET[0..32]
DATA_SECTION_OFFSET[32..64]
lw   $ds $is 1
add  $$ds $$ds $is
lw   $r0 $fp i73              ; load input function selector
lw   $r1 data_0               ; load fn selector for comparison
eq   $r2 $r0 $r1              ; function selector comparison
jnzi $r2 i12                  ; jump to selected function
movi $$tmp i123               ; special code for mismatched selector
rvrt $$tmp                    ; revert if no selectors matched
lw   $r0 $fp i74              ; base register for method parameter
addi $r0 $r0 i0               ; extract address
lw   $r0 $r0 i0               ; extract_value @ 0
ret  $r0
noop                          ; word-alignment of data section
.data:
data_0 .word 1230850595
