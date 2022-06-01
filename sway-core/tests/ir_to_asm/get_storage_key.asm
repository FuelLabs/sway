.program:
ji   i4
noop
DATA_SECTION_OFFSET[0..32]
DATA_SECTION_OFFSET[32..64]
lw   $ds $is 1
add  $$ds $$ds $is
lw   $r1 $fp i73              ; load input function selector
lw   $r0 data_3               ; load fn selector for comparison
eq   $r0 $r1 $r0              ; function selector comparison
jnzi $r0 i14                  ; jump to selected function
lw   $r0 data_4               ; load fn selector for comparison
eq   $r0 $r1 $r0              ; function selector comparison
jnzi $r0 i18                  ; jump to selected function
rvrt $zero                    ; revert if no selectors matched
lw   $r0 data_0               ; literal instantiation
lw   $r1 data_1               ; loading size for RETD
retd  $r0 $r1
lw   $r1 data_2               ; literal instantiation
lw   $r0 data_1               ; loading size for RETD
retd  $r1 $r0
noop                          ; word-alignment of data section
.data:
data_0 .b256 0xf383b0ce51358be57daa3b725fe44acdb2d880604e367199080b4379c41bb6ed
data_1 .u64 0x20
data_2 .b256 0xde9090cb50e71c2588c773487d1da7066d0c719849a7e58dc8b6397a25c567c0
data_3 .u32 0x2994c98e
data_4 .u32 0xf57bdec8
