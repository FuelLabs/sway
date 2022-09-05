.program:
ji   i4
noop
DATA_SECTION_OFFSET[0..32]
DATA_SECTION_OFFSET[32..64]
lw   $ds $is 1
add  $$ds $$ds $is
move $r2 $sp                  ; save locals base register
cfei i8                       ; allocate 8 bytes for all locals
addi $r0 $r2 i0               ; get offset reg for get_ptr
move $r1 $sp                  ; save register for temporary stack value
cfei i8                       ; allocate 8 bytes for temporary struct
lw   $r0 data_0               ; literal instantiation for aggregate field
sw   $r1 $r0 i0               ; initialise aggregate field
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
.data:
data_0 .u64 0x01
data_1 .u64 0x2a
data_2 .u64 0xf891e
data_3 .u64 0xf8923
data_4 .u64 0x08
