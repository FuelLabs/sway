.program:
ji   i4
noop
DATA_SECTION_OFFSET[0..32]
DATA_SECTION_OFFSET[32..64]
lw   $ds $is 1
add  $$ds $$ds $is
move $r2 $sp                  ; save locals base register
cfei i8                       ; allocate 8 bytes for locals
addi $r0 $r2 i0               ; get offset reg for get_ptr
lw   $r1 data_0               ; literal instantiation
addi $r0 $r2 i0               ; get store offset
mcpi $r0 $r1 i8               ; store value
lw   $r1 data_1               ; literal instantiation
lw   $r0 data_2               ; literal instantiation
log  $r1 $r0 $zero $zero
addi $r2 $r2 i0               ; get offset reg for get_ptr
lw   $r1 data_3               ; literal instantiation
lw   $r0 data_4               ; loading size for LOGD
logd $zero $r1 $r2 $r0
ret  $zero                    ; returning unit as zero
noop                          ; word-alignment of data section
.data:
data_0 .collection { .word 1 }
data_1 .word 42
data_2 .word 1018142
data_3 .word 1018147
data_4 .word 8
