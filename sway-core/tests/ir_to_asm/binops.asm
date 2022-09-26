.program:
ji   i4
noop
DATA_SECTION_OFFSET[0..32]
DATA_SECTION_OFFSET[32..64]
lw   $ds $is 1
add  $$ds $$ds $is
move $r2 $sp                  ; save locals base register
cfei i48                      ; allocate 48 bytes for all locals
addi $r0 $r2 i0               ; get offset reg for get_ptr
lw   $r0 data_0               ; literal instantiation
sw   $r2 $r0 i0               ; store value
addi $r0 $r2 i8               ; get offset reg for get_ptr
lw   $r0 data_1               ; literal instantiation
sw   $r2 $r0 i1               ; store value
addi $r0 $r2 i0               ; get offset reg for get_ptr
lw   $r1 $r2 i0               ; load value
addi $r0 $r2 i8               ; get offset reg for get_ptr
lw   $r0 $r2 i1               ; load value
add  $r0 $r1 $r0
addi $r1 $r2 i16              ; get offset reg for get_ptr
sw   $r2 $r0 i2               ; store value
addi $r0 $r2 i16              ; get offset reg for get_ptr
lw   $r1 $r2 i2               ; load value
lw   $r0 data_2               ; literal instantiation
mul  $r1 $r1 $r0
addi $r0 $r2 i24              ; get offset reg for get_ptr
sw   $r2 $r1 i3               ; store value
addi $r0 $r2 i24              ; get offset reg for get_ptr
lw   $r0 $r2 i3               ; load value
sub  $r1 $r0 $one
addi $r0 $r2 i32              ; get offset reg for get_ptr
sw   $r2 $r1 i4               ; store value
addi $r0 $r2 i32              ; get offset reg for get_ptr
lw   $r1 $r2 i4               ; load value
lw   $r0 data_2               ; literal instantiation
div  $r1 $r1 $r0
addi $r0 $r2 i40              ; get offset reg for get_ptr
sw   $r2 $r1 i5               ; store value
addi $r0 $r2 i40              ; get offset reg for get_ptr
lw   $r0 $r2 i5               ; load value
ret  $r0
.data:
data_0 .u64 0x16
data_1 .u64 0x2c
data_2 .u64 0x02
