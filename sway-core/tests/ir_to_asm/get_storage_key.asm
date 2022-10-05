.program:
ji   i4
noop
DATA_SECTION_OFFSET[0..32]
DATA_SECTION_OFFSET[32..64]
lw   $ds $is 1
add  $$ds $$ds $is
lw   $r0 $fp i73              ; load input function selector
lw   $r1 data_3               ; load fn selector for comparison
eq   $r2 $r0 $r1              ; function selector comparison
jnzi $r2 i15                  ; jump to selected function
lw   $r1 data_4               ; load fn selector for comparison
eq   $r2 $r0 $r1              ; function selector comparison
jnzi $r2 i19                  ; jump to selected function
movi $$tmp i123               ; special code for mismatched selector
rvrt $$tmp                    ; revert if no selectors matched
lw   $r1 data_0               ; literal instantiation
lw   $r0 data_1               ; loading size for RETD
retd  $r1 $r0
lw   $r1 data_2               ; literal instantiation
lw   $r0 data_1               ; loading size for RETD
retd  $r1 $r0
.data:
data_0 .bytes[32] f3 83 b0 ce 51 35 8b e5 7d aa 3b 72 5f e4 4a cd b2 d8 80 60 4e 36 71 99 08 0b 43 79 c4 1b b6 ed  ....Q5..}.;r_.J....`N6q...Cy....
data_1 .word 32
data_2 .bytes[32] de 90 90 cb 50 e7 1c 25 88 c7 73 48 7d 1d a7 06 6d 0c 71 98 49 a7 e5 8d c8 b6 39 7a 25 c5 67 c0  ....P..%..sH}...m.q.I.....9z%.g.
data_3 .word 697616782
data_4 .word 4118535880
