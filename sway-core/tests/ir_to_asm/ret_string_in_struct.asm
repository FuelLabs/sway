.program:
ji   i4
noop
DATA_SECTION_OFFSET[0..32]
DATA_SECTION_OFFSET[32..64]
lw   $ds $is 1
add  $$ds $$ds $is
lw   $r1 $fp i73              ; load input function selector
lw   $r0 data_4               ; load fn selector for comparison
eq   $r0 $r1 $r0              ; function selector comparison
jnzi $r0 i14                  ; jump to selected function
lw   $r0 data_5               ; load fn selector for comparison
eq   $r0 $r1 $r0              ; function selector comparison
jnzi $r0 i22                  ; jump to selected function
rvrt $zero                    ; revert if no selectors matched
move $r2 $sp                  ; save register for temporary stack value
cfei i8                       ; allocate 8 bytes for temporary struct
lw   $r1 data_0               ; literal instantiation
addi $r0 $r2 i0               ; get struct field(s) 0 offset
mcpi $r0 $r1 i8               ; store struct field value
lw   $r0 data_1               ; loading size for RETD
retd  $r2 $r0
move $r2 $sp                  ; save register for temporary stack value
cfei i16                      ; allocate 16 bytes for temporary struct
lw   $r1 data_2               ; literal instantiation
addi $r0 $r2 i0               ; get struct field(s) 0 offset
mcpi $r0 $r1 i16              ; store struct field value
lw   $r0 data_3               ; loading size for RETD
retd  $r2 $r0
noop                          ; word-alignment of data section
.data:
data_0 .str "foobar0"
data_1 .u64 0x08
data_2 .str "foobarbaz"
data_3 .u64 0x10
data_4 .u32 0x4a13be00
data_5 .u32 0x29ea7974
