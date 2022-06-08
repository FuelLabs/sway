.program:
ji   i4
noop
DATA_SECTION_OFFSET[0..32]
DATA_SECTION_OFFSET[32..64]
lw   $ds $is 1
add  $$ds $$ds $is
move $r1 $sp                  ; save register for temporary stack value
cfei i72                      ; allocate 72 bytes for temporary struct
lw   $r0 data_0               ; literal instantiation for aggregate field
sw   $r1 $r0 i0               ; initialise aggregate field
addi $r0 $r1 i8               ; get base pointer for union
mcli $r0 i16                  ; clear padding for union initialisation
lw   $r0 data_1               ; literal instantiation for aggregate field
sw   $r1 $r0 i3               ; initialise aggregate field
lw   $r0 data_0               ; literal instantiation for aggregate field
sw   $r1 $r0 i4               ; initialise aggregate field
addi $r0 $r1 i40              ; get base pointer for union
mcli $r0 i24                  ; clear padding for union initialisation
lw   $r0 data_2               ; literal instantiation for aggregate field
sw   $r1 $r0 i8               ; initialise aggregate field
lw   $r0 data_3               ; loading size for RETD
retd  $r1 $r0
noop                          ; word-alignment of data section
.data:
data_0 .u64 0x01
data_1 .u64 0x2a
data_2 .u64 0x42
data_3 .u64 0x48
