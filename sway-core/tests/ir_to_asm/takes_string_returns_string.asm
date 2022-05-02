.program:
ji   i4
noop
DATA_SECTION_OFFSET[0..32]
DATA_SECTION_OFFSET[32..64]
lw   $ds $is 1
add  $$ds $$ds $is
lw   $r1 $fp i73              ; load input function selector
lw   $r0 data_2               ; load fn selector for comparison
eq   $r0 $r1 $r0              ; function selector comparison
jnzi $r0 i14                  ; jump to selected function
lw   $r0 data_3               ; load fn selector for comparison
eq   $r0 $r1 $r0              ; function selector comparison
jnzi $r0 i17                  ; jump to selected function
rvrt $zero                    ; revert if no selectors matched
lw   $r1 $fp i74              ; Base register for method parameter
lw   $r0 data_0               ; loading size for RETD
retd  $r1 $r0
lw   $r1 $fp i74              ; Base register for method parameter
lw   $r0 data_1               ; loading size for RETD
retd  $r1 $r0
noop                          ; word-alignment of data section
.data:
data_0 .u64 0x08
data_1 .u64 0x10
data_2 .u32 0x80da70e2
data_3 .u32 0x28c0f699
