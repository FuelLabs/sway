.program:
ji   i4
noop
DATA_SECTION_OFFSET[0..32]
DATA_SECTION_OFFSET[32..64]
lw   $ds $is 1
add  $$ds $$ds $is
move $r2 $sp                  ; save locals base register
cfei i96                      ; allocate 96 bytes for all locals
lw   $r1 data_0               ; literal instantiation
addi $r0 $r2 i0               ; get offset reg for get_ptr
addi $r0 $r2 i0               ; get store offset
mcpi $r0 $r1 i32              ; store value
lw   $r0 data_1               ; literal instantiation
addi $r1 $r2 i32              ; get offset reg for get_ptr
addi $r1 $r2 i32              ; get store offset
mcpi $r1 $r0 i32              ; store value
lw   $r0 data_2               ; literal instantiation
addi $r1 $r2 i64              ; get offset reg for get_ptr
addi $r1 $r2 i64              ; get store offset
mcpi $r1 $r0 i32              ; store value
ret  $zero                    ; returning unit as zero
.data:
data_0 .b256 0xa11bfeaaf375e5527e56b335059ce8ba90b767d7b6abe24ad4752c28be4dbceb
data_1 .b256 0xb3014a9722208a5ad14317a887fdac3f153f92968d905cea638cf54db8f133e0
data_2 .b256 0x040d53c164d38731950be9278876db5fa5ab7409fdee8e3c87e2e6fbbeeebd48
