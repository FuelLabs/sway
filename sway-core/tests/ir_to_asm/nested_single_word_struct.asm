.program:
ji   i4
noop
DATA_SECTION_OFFSET[0..32]
DATA_SECTION_OFFSET[32..64]
lw   $ds $is 1
add  $$ds $$ds $is
lw   $r1 $fp i73              ; load input function selector
lw   $r0 data_0               ; load fn selector for comparison
eq   $r0 $r1 $r0              ; function selector comparison
jnei $zero $r0 i11            ; jump to selected function
rvrt $zero                    ; revert if no selectors matched
lw   $r0 $fp i74              ; Base register for method parameter
addi $r0 $r0 i0               ; extract address
lw   $r0 $r0 i0               ; extract_value @ 0
ret  $r0
.data:
data_0 .u32 0x495d4a23
