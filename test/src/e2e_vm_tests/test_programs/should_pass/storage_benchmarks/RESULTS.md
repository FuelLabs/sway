# Branch: `master` on 2026.03.26 `8e643df9de76cb6dbf7972382fed15f820230cff` (Support attributes and constants in self `impl Contract`s (#7580))

```console
═══════════════════════════════════════════════════════════════
  Running: storage_fields
═══════════════════════════════════════════════════════════════

--- CSV (baseline 12198 gas subtracted) ---
test,gas
bench_bool_read,854
bench_bool_write,16845
bench_bool_clear,17389
bench_u8_read,836
bench_u8_write,16865
bench_u8_clear,17409
bench_u16_read,860
bench_u16_write,16847
bench_u16_clear,17409
bench_u32_read,869
bench_u32_write,16874
bench_u32_clear,17436
bench_u64_read,878
bench_u64_write,16892
bench_u64_clear,17454
bench_u256_read,897
bench_u256_write,16123
bench_u256_clear,17417
bench_struct24_read,886
bench_struct24_write,16939
bench_struct24_clear,17497
bench_struct32_read,895
bench_struct32_write,16166
bench_struct32_clear,17460
bench_struct40_read,1216
bench_struct40_write,25443
bench_struct40_clear,26854
bench_struct48_read,1225
bench_struct48_write,25461
bench_struct48_clear,26872
bench_struct56_read,1234
bench_struct56_write,25488
bench_struct56_clear,26899
bench_struct72_read,1555
bench_struct72_write,33974
bench_struct72_clear,36238
bench_struct88_read,1564
bench_struct88_write,33992
bench_struct88_clear,36256
bench_struct96_read,1571
bench_struct96_write,32597
bench_struct96_clear,36274
bench_struct184_read,2452
bench_struct184_write,59260
bench_struct184_clear,64083
bench_struct200_read,2773
bench_struct200_write,67746
bench_struct200_clear,73422
bench_struct224_read,2782
bench_struct224_write,65103
bench_struct224_clear,73440
bench_struct552_read,6298
bench_struct552_write,160931
bench_struct552_clear,175989

--- Histogram (baseline 12198 gas subtracted) ---

  bool_read       │ █    854
  bool_write      │ █████  16845
  bool_clear      │ █████  17389
  u8_read         │ █    836
  u8_write        │ █████  16865
  u8_clear        │ █████  17409
  u16_read        │ █    860
  u16_write       │ █████  16847
  u16_clear       │ █████  17409
  u32_read        │ █    869
  u32_write       │ █████  16874
  u32_clear       │ █████  17436
  u64_read        │ █    878
  u64_write       │ █████  16892
  u64_clear       │ █████  17454
  u256_read       │ █    897
  u256_write      │ █████  16123
  u256_clear      │ █████  17417
  struct24_read   │ █    886
  struct24_write  │ █████  16939
  struct24_clear  │ █████  17497
  struct32_read   │ █    895
  struct32_write  │ █████  16166
  struct32_clear  │ █████  17460
  struct40_read   │ █   1216
  struct40_write  │ ████████  25443
  struct40_clear  │ █████████  26854
  struct48_read   │ █   1225
  struct48_write  │ ████████  25461
  struct48_clear  │ █████████  26872
  struct56_read   │ █   1234
  struct56_write  │ ████████  25488
  struct56_clear  │ █████████  26899
  struct72_read   │ █   1555
  struct72_write  │ ███████████  33974
  struct72_clear  │ ████████████  36238
  struct88_read   │ █   1564
  struct88_write  │ ███████████  33992
  struct88_clear  │ ████████████  36256
  struct96_read   │ █   1571
  struct96_write  │ ███████████  32597
  struct96_clear  │ ████████████  36274
  struct184_read  │ █   2452
  struct184_write │ ████████████████████  59260
  struct184_clear │ █████████████████████  64083
  struct200_read  │ █   2773
  struct200_write │ ███████████████████████  67746
  struct200_clear │ █████████████████████████  73422
  struct224_read  │ █   2782
  struct224_write │ ██████████████████████  65103
  struct224_clear │ █████████████████████████  73440
  struct552_read  │ ██   6298
  struct552_write │ ██████████████████████████████████████████████████████ 160931
  struct552_clear │ ████████████████████████████████████████████████████████████ 175989

═══════════════════════════════════════════════════════════════
  Running: storage_fields_partial_access
═══════════════════════════════════════════════════════════════

--- CSV (baseline 12022 gas subtracted) ---
test,gas
bench_struct24_read_u64,855
bench_struct24_write_u64,16846
bench_struct32_read_u64,866
bench_struct32_write_u64,16855
bench_struct40_read_u64,875
bench_struct40_write_u64,16864
bench_struct48_read_struct24,1210
bench_struct48_write_struct24,25421
bench_struct48_read_struct24_u64,915
bench_struct48_write_struct24_u64,16902
bench_struct56_read_struct24,907
bench_struct56_write_struct24,16971
bench_struct56_read_struct32,1228
bench_struct56_write_struct32,25454
bench_struct56_read_struct24_u64,924
bench_struct56_write_struct24_u64,16911
bench_struct56_read_struct32_u64,933
bench_struct56_write_struct32_u64,16920
bench_struct72_read_struct32,925
bench_struct72_write_struct32,16220
bench_struct72_read_struct40,1246
bench_struct72_write_struct40,25467
bench_struct72_read_struct32_u64,942
bench_struct72_write_struct32_u64,16929
bench_struct72_read_struct40_u64,951
bench_struct72_write_struct40_u64,16938
bench_struct88_read_struct40,1255
bench_struct88_write_struct40,25476
bench_struct88_read_u64,884
bench_struct88_write_u64,16882
bench_struct88_read_struct40_u64,958
bench_struct88_write_struct40_u64,16947
bench_struct96_read_struct48,1264
bench_struct96_write_struct48,25485
bench_struct96_read_struct48_struct24,954
bench_struct96_write_struct48_struct24,17000
bench_struct96_read_struct48_struct24_u64,969
bench_struct96_write_struct48_struct24_u64,16958
bench_struct184_read_struct96,1539
bench_struct184_write_struct96,32471
bench_struct184_read_struct88,1522
bench_struct184_write_struct88,33853
bench_struct184_read_struct96_struct48,1283
bench_struct184_write_struct96_struct48,25450
bench_struct184_read_struct88_struct40,1274
bench_struct184_write_struct88_struct40,25441
bench_struct184_read_struct96_struct48_struct24,1010
bench_struct184_write_struct96_struct48_struct24,17020
bench_struct184_read_struct88_struct40_u64,977
bench_struct184_write_struct88_struct40_u64,16957
bench_struct184_read_struct96_struct48_struct24_u64,1017
bench_struct184_write_struct96_struct48_struct24_u64,16997
bench_struct200_read_struct96,1550
bench_struct200_write_struct96,32480
bench_struct200_read_u64,865
bench_struct200_write_u64,16845
bench_struct200_read_struct96_struct48,1292
bench_struct200_write_struct96_struct48,25459
bench_struct200_read_struct96_struct48_struct24,1019
bench_struct200_write_struct96_struct48_struct24,17029
bench_struct200_read_struct96_struct48_struct24_u64,1026
bench_struct200_write_struct96_struct48_struct24_u64,17006
bench_struct224_read_struct96,1568
bench_struct224_write_struct96,32498
bench_struct224_read_struct32,933
bench_struct224_write_struct32,16174
bench_struct224_read_struct96_struct48,1301
bench_struct224_write_struct96_struct48,25468
bench_struct224_read_struct32_u64,921
bench_struct224_write_struct32_u64,16901
bench_struct224_read_struct96_struct48_struct24,1028
bench_struct224_write_struct96_struct48_struct24,17038
bench_struct224_read_struct96_struct48_struct24_u64,1035
bench_struct224_write_struct96_struct48_struct24_u64,17015
bench_struct552_read_struct224,2825
bench_struct552_write_struct224,65074
bench_struct552_read_struct96,1586
bench_struct552_write_struct96,32516
bench_struct552_read_u64,903
bench_struct552_write_u64,16854
bench_struct552_read_struct224_struct96,1641
bench_struct552_write_struct224_struct96,32535
bench_struct552_read_struct224_struct32,1006
bench_struct552_write_struct224_struct32,16211
bench_struct552_read_struct224_struct96_struct48,1357
bench_struct552_write_struct224_struct96_struct48,25497
bench_struct552_read_struct224_struct96_struct48_struct24,1050
bench_struct552_write_struct224_struct96_struct48_struct24,17060
bench_struct552_read_struct224_struct96_struct48_struct24_u64,1049
bench_struct552_write_struct224_struct96_struct48_struct24_u64,17029

--- Histogram (baseline 12022 gas subtracted) ---

  struct24_read_u64                                        │ █   855
  struct24_write_u64                                       │ ███████████████ 16846
  struct32_read_u64                                        │ █   866
  struct32_write_u64                                       │ ███████████████ 16855
  struct40_read_u64                                        │ █   875
  struct40_write_u64                                       │ ███████████████ 16864
  struct48_read_struct24                                   │ █  1210
  struct48_write_struct24                                  │ ███████████████████████ 25421
  struct48_read_struct24_u64                               │ █   915
  struct48_write_struct24_u64                              │ ███████████████ 16902
  struct56_read_struct24                                   │ █   907
  struct56_write_struct24                                  │ ███████████████ 16971
  struct56_read_struct32                                   │ █  1228
  struct56_write_struct32                                  │ ███████████████████████ 25454
  struct56_read_struct24_u64                               │ █   924
  struct56_write_struct24_u64                              │ ███████████████ 16911
  struct56_read_struct32_u64                               │ █   933
  struct56_write_struct32_u64                              │ ███████████████ 16920
  struct72_read_struct32                                   │ █   925
  struct72_write_struct32                                  │ ██████████████ 16220
  struct72_read_struct40                                   │ █  1246
  struct72_write_struct40                                  │ ███████████████████████ 25467
  struct72_read_struct32_u64                               │ █   942
  struct72_write_struct32_u64                              │ ███████████████ 16929
  struct72_read_struct40_u64                               │ █   951
  struct72_write_struct40_u64                              │ ███████████████ 16938
  struct88_read_struct40                                   │ █  1255
  struct88_write_struct40                                  │ ███████████████████████ 25476
  struct88_read_u64                                        │ █   884
  struct88_write_u64                                       │ ███████████████ 16882
  struct88_read_struct40_u64                               │ █   958
  struct88_write_struct40_u64                              │ ███████████████ 16947
  struct96_read_struct48                                   │ █  1264
  struct96_write_struct48                                  │ ███████████████████████ 25485
  struct96_read_struct48_struct24                          │ █   954
  struct96_write_struct48_struct24                         │ ███████████████ 17000
  struct96_read_struct48_struct24_u64                      │ █   969
  struct96_write_struct48_struct24_u64                     │ ███████████████ 16958
  struct184_read_struct96                                  │ █  1539
  struct184_write_struct96                                 │ █████████████████████████████ 32471
  struct184_read_struct88                                  │ █  1522
  struct184_write_struct88                                 │ ███████████████████████████████ 33853
  struct184_read_struct96_struct48                         │ █  1283
  struct184_write_struct96_struct48                        │ ███████████████████████ 25450
  struct184_read_struct88_struct40                         │ █  1274
  struct184_write_struct88_struct40                        │ ███████████████████████ 25441
  struct184_read_struct96_struct48_struct24                │ █  1010
  struct184_write_struct96_struct48_struct24               │ ███████████████ 17020
  struct184_read_struct88_struct40_u64                     │ █   977
  struct184_write_struct88_struct40_u64                    │ ███████████████ 16957
  struct184_read_struct96_struct48_struct24_u64            │ █  1017
  struct184_write_struct96_struct48_struct24_u64           │ ███████████████ 16997
  struct200_read_struct96                                  │ █  1550
  struct200_write_struct96                                 │ █████████████████████████████ 32480
  struct200_read_u64                                       │ █   865
  struct200_write_u64                                      │ ███████████████ 16845
  struct200_read_struct96_struct48                         │ █  1292
  struct200_write_struct96_struct48                        │ ███████████████████████ 25459
  struct200_read_struct96_struct48_struct24                │ █  1019
  struct200_write_struct96_struct48_struct24               │ ███████████████ 17029
  struct200_read_struct96_struct48_struct24_u64            │ █  1026
  struct200_write_struct96_struct48_struct24_u64           │ ███████████████ 17006
  struct224_read_struct96                                  │ █  1568
  struct224_write_struct96                                 │ █████████████████████████████ 32498
  struct224_read_struct32                                  │ █   933
  struct224_write_struct32                                 │ ██████████████ 16174
  struct224_read_struct96_struct48                         │ █  1301
  struct224_write_struct96_struct48                        │ ███████████████████████ 25468
  struct224_read_struct32_u64                              │ █   921
  struct224_write_struct32_u64                             │ ███████████████ 16901
  struct224_read_struct96_struct48_struct24                │ █  1028
  struct224_write_struct96_struct48_struct24               │ ███████████████ 17038
  struct224_read_struct96_struct48_struct24_u64            │ █  1035
  struct224_write_struct96_struct48_struct24_u64           │ ███████████████ 17015
  struct552_read_struct224                                 │ ██  2825
  struct552_write_struct224                                │ ████████████████████████████████████████████████████████████ 65074
  struct552_read_struct96                                  │ █  1586
  struct552_write_struct96                                 │ █████████████████████████████ 32516
  struct552_read_u64                                       │ █   903
  struct552_write_u64                                      │ ███████████████ 16854
  struct552_read_struct224_struct96                        │ █  1641
  struct552_write_struct224_struct96                       │ █████████████████████████████ 32535
  struct552_read_struct224_struct32                        │ █  1006
  struct552_write_struct224_struct32                       │ ██████████████ 16211
  struct552_read_struct224_struct96_struct48               │ █  1357
  struct552_write_struct224_struct96_struct48              │ ███████████████████████ 25497
  struct552_read_struct224_struct96_struct48_struct24      │ █  1050
  struct552_write_struct224_struct96_struct48_struct24     │ ███████████████ 17060
  struct552_read_struct224_struct96_struct48_struct24_u64  │ █  1049
  struct552_write_struct224_struct96_struct48_struct24_u64 │ ███████████████ 17029
```
