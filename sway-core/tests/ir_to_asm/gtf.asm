.program:
ji   i4
noop
DATA_SECTION_OFFSET[0..32]
DATA_SECTION_OFFSET[32..64]
lw   $ds $is 1
add  $$ds $$ds $is
gtf  $r0 $zero i1             ; get transaction field
ret  $r0
noop                          ; word-alignment of data section
.data:
