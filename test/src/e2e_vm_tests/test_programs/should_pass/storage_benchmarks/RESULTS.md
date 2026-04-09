# Branch: `master` on 2026.04.08 `551e37f20aa44e49f2520e333e2934b3d209f820` (Pin tracing-subscriber to 0.3.19 (#7587))

```text
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

═══════════════════════════════════════════════════════════════
  Running: storage_vec_s8
═══════════════════════════════════════════════════════════════

--- CSV (per-count baselines subtracted) ---
  empty-call baseline: 11947
  populate baselines : n10=376397  n100=3599629  n1000=35852037  
  store_vec baselines: n10=12894  n100=18345  n1000=71617  
test,gas
bench_push_n10,34859
bench_push_n100,38906
bench_push_n1000,38884
bench_push_n_elems_into_empty_vec_n10,364538
bench_push_n_elems_into_empty_vec_n100,3587770
bench_push_n_elems_into_empty_vec_n1000,35840178
bench_pop_n10,18840
bench_pop_n100,18902
bench_pop_n1000,18913
bench_get_n10,2697
bench_get_n100,2739
bench_get_n1000,2752
bench_set_n10,18044
bench_set_n100,18097
bench_set_n1000,18110
bench_first_n10,2741
bench_first_n100,2741
bench_first_n1000,2730
bench_last_n10,2759
bench_last_n100,2772
bench_last_n1000,2772
bench_len_n10,841
bench_len_n100,903
bench_len_n1000,916
bench_is_empty_n10,913
bench_is_empty_n100,902
bench_is_empty_n1000,902
bench_swap_n10,36621
bench_swap_n100,36634
bench_swap_n1000,36612
bench_swap_remove_n10,36612
bench_swap_remove_n100,36603
bench_swap_remove_n1000,36614
bench_remove_n10,89627
bench_remove_n100,884328
bench_remove_n1000,8831789
bench_insert_n10,123218
bench_insert_n100,922209
bench_insert_n1000,8871898
bench_reverse_n10,177852
bench_reverse_n100,1766813
bench_reverse_n1000,17656291
bench_fill_n10,169253
bench_fill_n100,1681527
bench_fill_n1000,16804227
bench_resize_grow_n10,194195
bench_resize_grow_n100,1799381
bench_resize_grow_n1000,17831081
bench_resize_shrink_n10,18108
bench_resize_shrink_n100,18097
bench_resize_shrink_n1000,18097
bench_store_vec_n10,65810
bench_store_vec_n100,333921
bench_store_vec_n1000,3076199
bench_load_vec_n10,2812
bench_load_vec_n100,9667
bench_load_vec_n1000,79902
bench_iter_n10,20698
bench_iter_n100,198011
bench_iter_n1000,1971011
bench_clear_n10,17353
bench_clear_n100,17353
bench_clear_n1000,17353

--- Histogram ---

  push_n10                          │ █    34859
  push_n100                         │ █    38906
  push_n1000                        │ █    38884
  push_n_elems_into_empty_vec_n10   │ █   364538
  push_n_elems_into_empty_vec_n100  │ ██████  3587770
  push_n_elems_into_empty_vec_n1000 │ ████████████████████████████████████████████████████████████ 35840178
  pop_n10                           │ █    18840
  pop_n100                          │ █    18902
  pop_n1000                         │ █    18913
  get_n10                           │ █     2697
  get_n100                          │ █     2739
  get_n1000                         │ █     2752
  set_n10                           │ █    18044
  set_n100                          │ █    18097
  set_n1000                         │ █    18110
  first_n10                         │ █     2741
  first_n100                        │ █     2741
  first_n1000                       │ █     2730
  last_n10                          │ █     2759
  last_n100                         │ █     2772
  last_n1000                        │ █     2772
  len_n10                           │ █      841
  len_n100                          │ █      903
  len_n1000                         │ █      916
  is_empty_n10                      │ █      913
  is_empty_n100                     │ █      902
  is_empty_n1000                    │ █      902
  swap_n10                          │ █    36621
  swap_n100                         │ █    36634
  swap_n1000                        │ █    36612
  swap_remove_n10                   │ █    36612
  swap_remove_n100                  │ █    36603
  swap_remove_n1000                 │ █    36614
  remove_n10                        │ █    89627
  remove_n100                       │ █   884328
  remove_n1000                      │ ██████████████  8831789
  insert_n10                        │ █   123218
  insert_n100                       │ █   922209
  insert_n1000                      │ ██████████████  8871898
  reverse_n10                       │ █   177852
  reverse_n100                      │ ██  1766813
  reverse_n1000                     │ █████████████████████████████ 17656291
  fill_n10                          │ █   169253
  fill_n100                         │ ██  1681527
  fill_n1000                        │ ████████████████████████████ 16804227
  resize_grow_n10                   │ █   194195
  resize_grow_n100                  │ ███  1799381
  resize_grow_n1000                 │ █████████████████████████████ 17831081
  resize_shrink_n10                 │ █    18108
  resize_shrink_n100                │ █    18097
  resize_shrink_n1000               │ █    18097
  store_vec_n10                     │ █    65810
  store_vec_n100                    │ █   333921
  store_vec_n1000                   │ █████  3076199
  load_vec_n10                      │ █     2812
  load_vec_n100                     │ █     9667
  load_vec_n1000                    │ █    79902
  iter_n10                          │ █    20698
  iter_n100                         │ █   198011
  iter_n1000                        │ ███  1971011
  clear_n10                         │ █    17353
  clear_n100                        │ █    17353
  clear_n1000                       │ █    17353

═══════════════════════════════════════════════════════════════
  Running: storage_vec_s24
═══════════════════════════════════════════════════════════════

--- CSV (per-count baselines subtracted) ---
  empty-call baseline: 12233
  populate baselines : n10=439783  n100=4230915  n1000=42162323  
  store_vec baselines: n10=13882  n100=25641  n1000=141980  
test,gas
bench_push_n10,47421
bench_push_n100,38966
bench_push_n1000,38944
bench_push_n_elems_into_empty_vec_n10,427638
bench_push_n_elems_into_empty_vec_n100,4218770
bench_push_n_elems_into_empty_vec_n1000,42150178
bench_pop_n10,19190
bench_pop_n100,18920
bench_pop_n1000,18933
bench_get_n10,3045
bench_get_n100,3067
bench_get_n1000,2770
bench_set_n10,26582
bench_set_n100,26635
bench_set_n1000,18180
bench_first_n10,2763
bench_first_n100,2761
bench_first_n1000,2750
bench_last_n10,3087
bench_last_n100,2790
bench_last_n1000,2788
bench_len_n10,861
bench_len_n100,903
bench_len_n1000,916
bench_is_empty_n10,913
bench_is_empty_n100,902
bench_is_empty_n1000,902
bench_swap_n10,45547
bench_swap_n100,36780
bench_swap_n1000,36758
bench_swap_remove_n10,45810
bench_swap_remove_n100,45489
bench_swap_remove_n1000,36720
bench_remove_n10,107820
bench_remove_n100,1107434
bench_remove_n1000,11063245
bench_insert_n10,162495
bench_insert_n100,1145431
bench_insert_n1000,11103470
bench_reverse_n10,222482
bench_reverse_n100,2213113
bench_reverse_n1000,22119291
bench_fill_n10,212111
bench_fill_n100,2109943
bench_fill_n1000,21088243
bench_resize_grow_n10,257215
bench_resize_grow_n100,2429401
bench_resize_grow_n1000,24131101
bench_resize_shrink_n10,18128
bench_resize_shrink_n100,18117
bench_resize_shrink_n1000,18117
bench_store_vec_n10,126728
bench_store_vec_n100,943119
bench_store_vec_n1000,9168199
bench_load_vec_n10,4372
bench_load_vec_n100,25275
bench_load_vec_n1000,235978
bench_iter_n10,22418
bench_iter_n100,215411
bench_iter_n1000,2143011
bench_clear_n10,17354
bench_clear_n100,17354
bench_clear_n1000,17354

--- Histogram ---

  push_n10                          │ █    47421
  push_n100                         │ █    38966
  push_n1000                        │ █    38944
  push_n_elems_into_empty_vec_n10   │ █   427638
  push_n_elems_into_empty_vec_n100  │ ██████  4218770
  push_n_elems_into_empty_vec_n1000 │ ████████████████████████████████████████████████████████████ 42150178
  pop_n10                           │ █    19190
  pop_n100                          │ █    18920
  pop_n1000                         │ █    18933
  get_n10                           │ █     3045
  get_n100                          │ █     3067
  get_n1000                         │ █     2770
  set_n10                           │ █    26582
  set_n100                          │ █    26635
  set_n1000                         │ █    18180
  first_n10                         │ █     2763
  first_n100                        │ █     2761
  first_n1000                       │ █     2750
  last_n10                          │ █     3087
  last_n100                         │ █     2790
  last_n1000                        │ █     2788
  len_n10                           │ █      861
  len_n100                          │ █      903
  len_n1000                         │ █      916
  is_empty_n10                      │ █      913
  is_empty_n100                     │ █      902
  is_empty_n1000                    │ █      902
  swap_n10                          │ █    45547
  swap_n100                         │ █    36780
  swap_n1000                        │ █    36758
  swap_remove_n10                   │ █    45810
  swap_remove_n100                  │ █    45489
  swap_remove_n1000                 │ █    36720
  remove_n10                        │ █   107820
  remove_n100                       │ █  1107434
  remove_n1000                      │ ███████████████ 11063245
  insert_n10                        │ █   162495
  insert_n100                       │ █  1145431
  insert_n1000                      │ ███████████████ 11103470
  reverse_n10                       │ █   222482
  reverse_n100                      │ ███  2213113
  reverse_n1000                     │ ███████████████████████████████ 22119291
  fill_n10                          │ █   212111
  fill_n100                         │ ███  2109943
  fill_n1000                        │ ██████████████████████████████ 21088243
  resize_grow_n10                   │ █   257215
  resize_grow_n100                  │ ███  2429401
  resize_grow_n1000                 │ ██████████████████████████████████ 24131101
  resize_shrink_n10                 │ █    18128
  resize_shrink_n100                │ █    18117
  resize_shrink_n1000               │ █    18117
  store_vec_n10                     │ █   126728
  store_vec_n100                    │ █   943119
  store_vec_n1000                   │ █████████████  9168199
  load_vec_n10                      │ █     4372
  load_vec_n100                     │ █    25275
  load_vec_n1000                    │ █   235978
  iter_n10                          │ █    22418
  iter_n100                         │ █   215411
  iter_n1000                        │ ███  2143011
  clear_n10                         │ █    17354
  clear_n100                        │ █    17354
  clear_n1000                       │ █    17354

═══════════════════════════════════════════════════════════════
  Running: storage_vec_s32
═══════════════════════════════════════════════════════════════

--- CSV (per-count baselines subtracted) ---
  empty-call baseline: 12243
  populate baselines : n10=404781  n100=3908129  n1000=38941537  
  store_vec baselines: n10=13894  n100=25658  n1000=142034  
test,gas
bench_push_n10,38959
bench_push_n100,38972
bench_push_n1000,38950
bench_push_n_elems_into_empty_vec_n10,392626
bench_push_n_elems_into_empty_vec_n100,3895974
bench_push_n_elems_into_empty_vec_n1000,38929382
bench_pop_n10,18878
bench_pop_n100,18920
bench_pop_n1000,18933
bench_get_n10,2733
bench_get_n100,2755
bench_get_n1000,2770
bench_set_n10,18120
bench_set_n100,18173
bench_set_n1000,18186
bench_first_n10,2764
bench_first_n100,2762
bench_first_n1000,2751
bench_last_n10,2777
bench_last_n100,2790
bench_last_n1000,2788
bench_len_n10,861
bench_len_n100,903
bench_len_n1000,916
bench_is_empty_n10,913
bench_is_empty_n100,902
bench_is_empty_n1000,902
bench_swap_n10,35983
bench_swap_n100,35996
bench_swap_n1000,35974
bench_swap_remove_n10,36726
bench_swap_remove_n100,36717
bench_swap_remove_n1000,36728
bench_remove_n10,89972
bench_remove_n100,888228
bench_remove_n1000,8871239
bench_insert_n10,127723
bench_insert_n100,926237
bench_insert_n1000,8911476
bench_reverse_n10,177846
bench_reverse_n100,1773917
bench_reverse_n1000,17734495
bench_fill_n10,169035
bench_fill_n100,1686347
bench_fill_n1000,16859447
bench_resize_grow_n10,227031
bench_resize_grow_n100,2107401
bench_resize_grow_n1000,20911101
bench_resize_shrink_n10,18128
bench_resize_shrink_n100,18117
bench_resize_shrink_n1000,18117
bench_store_vec_n10,151090
bench_store_vec_n100,1247819
bench_store_vec_n1000,12215199
bench_load_vec_n10,4996
bench_load_vec_n100,33078
bench_load_vec_n1000,314018
bench_iter_n10,20858
bench_iter_n100,199811
bench_iter_n1000,1987011
bench_clear_n10,17354
bench_clear_n100,17354
bench_clear_n1000,17354

--- Histogram ---

  push_n10                          │ █    38959
  push_n100                         │ █    38972
  push_n1000                        │ █    38950
  push_n_elems_into_empty_vec_n10   │ █   392626
  push_n_elems_into_empty_vec_n100  │ ██████  3895974
  push_n_elems_into_empty_vec_n1000 │ ████████████████████████████████████████████████████████████ 38929382
  pop_n10                           │ █    18878
  pop_n100                          │ █    18920
  pop_n1000                         │ █    18933
  get_n10                           │ █     2733
  get_n100                          │ █     2755
  get_n1000                         │ █     2770
  set_n10                           │ █    18120
  set_n100                          │ █    18173
  set_n1000                         │ █    18186
  first_n10                         │ █     2764
  first_n100                        │ █     2762
  first_n1000                       │ █     2751
  last_n10                          │ █     2777
  last_n100                         │ █     2790
  last_n1000                        │ █     2788
  len_n10                           │ █      861
  len_n100                          │ █      903
  len_n1000                         │ █      916
  is_empty_n10                      │ █      913
  is_empty_n100                     │ █      902
  is_empty_n1000                    │ █      902
  swap_n10                          │ █    35983
  swap_n100                         │ █    35996
  swap_n1000                        │ █    35974
  swap_remove_n10                   │ █    36726
  swap_remove_n100                  │ █    36717
  swap_remove_n1000                 │ █    36728
  remove_n10                        │ █    89972
  remove_n100                       │ █   888228
  remove_n1000                      │ █████████████  8871239
  insert_n10                        │ █   127723
  insert_n100                       │ █   926237
  insert_n1000                      │ █████████████  8911476
  reverse_n10                       │ █   177846
  reverse_n100                      │ ██  1773917
  reverse_n1000                     │ ███████████████████████████ 17734495
  fill_n10                          │ █   169035
  fill_n100                         │ ██  1686347
  fill_n1000                        │ █████████████████████████ 16859447
  resize_grow_n10                   │ █   227031
  resize_grow_n100                  │ ███  2107401
  resize_grow_n1000                 │ ████████████████████████████████ 20911101
  resize_shrink_n10                 │ █    18128
  resize_shrink_n100                │ █    18117
  resize_shrink_n1000               │ █    18117
  store_vec_n10                     │ █   151090
  store_vec_n100                    │ █  1247819
  store_vec_n1000                   │ ██████████████████ 12215199
  load_vec_n10                      │ █     4996
  load_vec_n100                     │ █    33078
  load_vec_n1000                    │ █   314018
  iter_n10                          │ █    20858
  iter_n100                         │ █   199811
  iter_n1000                        │ ███  1987011
  clear_n10                         │ █    17354
  clear_n100                        │ █    17354
  clear_n1000                       │ █    17354

═══════════════════════════════════════════════════════════════
  Running: storage_vec_s56
═══════════════════════════════════════════════════════════════

--- CSV (per-count baselines subtracted) ---
  empty-call baseline: 12252
  populate baselines : n10=564802  n100=5480934  n1000=54662342  
  store_vec baselines: n10=13904  n100=25682  n1000=142160  
test,gas
bench_push_n10,59921
bench_push_n100,51466
bench_push_n1000,51444
bench_push_n_elems_into_empty_vec_n10,552638
bench_push_n_elems_into_empty_vec_n100,5468770
bench_push_n_elems_into_empty_vec_n1000,54650178
bench_pop_n10,19502
bench_pop_n100,19232
bench_pop_n1000,19245
bench_get_n10,3357
bench_get_n100,3379
bench_get_n1000,3082
bench_set_n10,35050
bench_set_n100,35103
bench_set_n1000,26648
bench_first_n10,3077
bench_first_n100,3075
bench_first_n1000,3064
bench_last_n10,3403
bench_last_n100,3104
bench_last_n1000,3102
bench_len_n10,861
bench_len_n100,903
bench_len_n1000,916
bench_is_empty_n10,913
bench_is_empty_n100,902
bench_is_empty_n1000,902
bench_swap_n10,63107
bench_swap_n100,54340
bench_swap_n1000,54318
bench_swap_remove_n10,54904
bench_swap_remove_n100,54583
bench_swap_remove_n1000,45814
bench_remove_n10,143254
bench_remove_n100,1537968
bench_remove_n1000,15444779
bench_insert_n10,218895
bench_insert_n100,1596931
bench_insert_n1000,15505970
bench_reverse_n10,310282
bench_reverse_n100,3091113
bench_reverse_n1000,30899291
bench_fill_n10,296791
bench_fill_n100,2956743
bench_fill_n1000,29556243
bench_resize_grow_n10,382215
bench_resize_grow_n100,3679401
bench_resize_grow_n1000,36631101
bench_resize_shrink_n10,18128
bench_resize_shrink_n100,18117
bench_resize_shrink_n1000,18117
bench_store_vec_n10,248609
bench_store_vec_n100,2161919
bench_store_vec_n1000,21356199
bench_load_vec_n10,7495
bench_load_vec_n100,56491
bench_load_vec_n1000,548133
bench_iter_n10,25558
bench_iter_n100,246611
bench_iter_n1000,2455011
bench_clear_n10,17354
bench_clear_n100,17354
bench_clear_n1000,17354

--- Histogram ---

  push_n10                          │ █    59921
  push_n100                         │ █    51466
  push_n1000                        │ █    51444
  push_n_elems_into_empty_vec_n10   │ █   552638
  push_n_elems_into_empty_vec_n100  │ ██████  5468770
  push_n_elems_into_empty_vec_n1000 │ ████████████████████████████████████████████████████████████ 54650178
  pop_n10                           │ █    19502
  pop_n100                          │ █    19232
  pop_n1000                         │ █    19245
  get_n10                           │ █     3357
  get_n100                          │ █     3379
  get_n1000                         │ █     3082
  set_n10                           │ █    35050
  set_n100                          │ █    35103
  set_n1000                         │ █    26648
  first_n10                         │ █     3077
  first_n100                        │ █     3075
  first_n1000                       │ █     3064
  last_n10                          │ █     3403
  last_n100                         │ █     3104
  last_n1000                        │ █     3102
  len_n10                           │ █      861
  len_n100                          │ █      903
  len_n1000                         │ █      916
  is_empty_n10                      │ █      913
  is_empty_n100                     │ █      902
  is_empty_n1000                    │ █      902
  swap_n10                          │ █    63107
  swap_n100                         │ █    54340
  swap_n1000                        │ █    54318
  swap_remove_n10                   │ █    54904
  swap_remove_n100                  │ █    54583
  swap_remove_n1000                 │ █    45814
  remove_n10                        │ █   143254
  remove_n100                       │ █  1537968
  remove_n1000                      │ ████████████████ 15444779
  insert_n10                        │ █   218895
  insert_n100                       │ █  1596931
  insert_n1000                      │ █████████████████ 15505970
  reverse_n10                       │ █   310282
  reverse_n100                      │ ███  3091113
  reverse_n1000                     │ █████████████████████████████████ 30899291
  fill_n10                          │ █   296791
  fill_n100                         │ ███  2956743
  fill_n1000                        │ ████████████████████████████████ 29556243
  resize_grow_n10                   │ █   382215
  resize_grow_n100                  │ ████  3679401
  resize_grow_n1000                 │ ████████████████████████████████████████ 36631101
  resize_shrink_n10                 │ █    18128
  resize_shrink_n100                │ █    18117
  resize_shrink_n1000               │ █    18117
  store_vec_n10                     │ █   248609
  store_vec_n100                    │ ██  2161919
  store_vec_n1000                   │ ███████████████████████ 21356199
  load_vec_n10                      │ █     7495
  load_vec_n100                     │ █    56491
  load_vec_n1000                    │ █   548133
  iter_n10                          │ █    25558
  iter_n100                         │ █   246611
  iter_n1000                        │ ██  2455011
  clear_n10                         │ █    17354
  clear_n100                        │ █    17354
  clear_n1000                       │ █    17354

═══════════════════════════════════════════════════════════════
  Running: storage_vec_s72
═══════════════════════════════════════════════════════════════

--- CSV (per-count baselines subtracted) ---
  empty-call baseline: 12260
  populate baselines : n10=627310  n100=6105942  n1000=60912350  
  store_vec baselines: n10=13914  n100=25700  n1000=142246  
test,gas
bench_push_n10,59921
bench_push_n100,63966
bench_push_n1000,63944
bench_push_n_elems_into_empty_vec_n10,615138
bench_push_n_elems_into_empty_vec_n100,6093770
bench_push_n_elems_into_empty_vec_n1000,60900178
bench_pop_n10,19502
bench_pop_n100,19544
bench_pop_n1000,19557
bench_get_n10,3357
bench_get_n100,3379
bench_get_n1000,3394
bench_set_n10,35050
bench_set_n100,35103
bench_set_n1000,35116
bench_first_n10,3389
bench_first_n100,3387
bench_first_n1000,3376
bench_last_n10,3403
bench_last_n100,3416
bench_last_n1000,3414
bench_len_n10,861
bench_len_n100,903
bench_len_n1000,916
bench_is_empty_n10,913
bench_is_empty_n100,902
bench_is_empty_n1000,902
bench_swap_n10,71887
bench_swap_n100,71900
bench_swap_n1000,71878
bench_swap_remove_n10,54904
bench_swap_remove_n100,54895
bench_swap_remove_n1000,54906
bench_remove_n10,160814
bench_remove_n100,1749000
bench_remove_n1000,17631311
bench_insert_n10,236455
bench_insert_n100,1828931
bench_insert_n1000,17713470
bench_reverse_n10,354182
bench_reverse_n100,3530113
bench_reverse_n1000,35289291
bench_fill_n10,339131
bench_fill_n100,3380143
bench_fill_n1000,33790243
bench_resize_grow_n10,444715
bench_resize_grow_n100,4304401
bench_resize_grow_n1000,42881101
bench_resize_shrink_n10,18128
bench_resize_shrink_n100,18117
bench_resize_shrink_n1000,18117
bench_store_vec_n10,309549
bench_store_vec_n100,2771319
bench_store_vec_n1000,27450199
bench_load_vec_n10,9056
bench_load_vec_n100,72099
bench_load_vec_n1000,704210
bench_iter_n10,27142
bench_iter_n100,262415
bench_iter_n1000,2613015
bench_clear_n10,17354
bench_clear_n100,17354
bench_clear_n1000,17354

--- Histogram ---

  push_n10                          │ █    59921
  push_n100                         │ █    63966
  push_n1000                        │ █    63944
  push_n_elems_into_empty_vec_n10   │ █   615138
  push_n_elems_into_empty_vec_n100  │ ██████  6093770
  push_n_elems_into_empty_vec_n1000 │ ████████████████████████████████████████████████████████████ 60900178
  pop_n10                           │ █    19502
  pop_n100                          │ █    19544
  pop_n1000                         │ █    19557
  get_n10                           │ █     3357
  get_n100                          │ █     3379
  get_n1000                         │ █     3394
  set_n10                           │ █    35050
  set_n100                          │ █    35103
  set_n1000                         │ █    35116
  first_n10                         │ █     3389
  first_n100                        │ █     3387
  first_n1000                       │ █     3376
  last_n10                          │ █     3403
  last_n100                         │ █     3416
  last_n1000                        │ █     3414
  len_n10                           │ █      861
  len_n100                          │ █      903
  len_n1000                         │ █      916
  is_empty_n10                      │ █      913
  is_empty_n100                     │ █      902
  is_empty_n1000                    │ █      902
  swap_n10                          │ █    71887
  swap_n100                         │ █    71900
  swap_n1000                        │ █    71878
  swap_remove_n10                   │ █    54904
  swap_remove_n100                  │ █    54895
  swap_remove_n1000                 │ █    54906
  remove_n10                        │ █   160814
  remove_n100                       │ █  1749000
  remove_n1000                      │ █████████████████ 17631311
  insert_n10                        │ █   236455
  insert_n100                       │ █  1828931
  insert_n1000                      │ █████████████████ 17713470
  reverse_n10                       │ █   354182
  reverse_n100                      │ ███  3530113
  reverse_n1000                     │ ██████████████████████████████████ 35289291
  fill_n10                          │ █   339131
  fill_n100                         │ ███  3380143
  fill_n1000                        │ █████████████████████████████████ 33790243
  resize_grow_n10                   │ █   444715
  resize_grow_n100                  │ ████  4304401
  resize_grow_n1000                 │ ██████████████████████████████████████████ 42881101
  resize_shrink_n10                 │ █    18128
  resize_shrink_n100                │ █    18117
  resize_shrink_n1000               │ █    18117
  store_vec_n10                     │ █   309549
  store_vec_n100                    │ ██  2771319
  store_vec_n1000                   │ ███████████████████████████ 27450199
  load_vec_n10                      │ █     9056
  load_vec_n100                     │ █    72099
  load_vec_n1000                    │ █   704210
  iter_n10                          │ █    27142
  iter_n100                         │ █   262415
  iter_n1000                        │ ██  2613015
  clear_n10                         │ █    17354
  clear_n100                        │ █    17354
  clear_n1000                       │ █    17354

═══════════════════════════════════════════════════════════════
  Running: storage_vec_s88
═══════════════════════════════════════════════════════════════

--- CSV (per-count baselines subtracted) ---
  empty-call baseline: 12267
  populate baselines : n10=689817  n100=6730949  n1000=67162357  
  store_vec baselines: n10=13921  n100=25716  n1000=142331  
test,gas
bench_push_n10,72421
bench_push_n100,63966
bench_push_n1000,63944
bench_push_n_elems_into_empty_vec_n10,677638
bench_push_n_elems_into_empty_vec_n100,6718770
bench_push_n_elems_into_empty_vec_n1000,67150178
bench_pop_n10,19817
bench_pop_n100,19547
bench_pop_n1000,19560
bench_get_n10,3670
bench_get_n100,3694
bench_get_n1000,3397
bench_set_n10,43518
bench_set_n100,43571
bench_set_n1000,35116
bench_first_n10,3390
bench_first_n100,3388
bench_first_n1000,3377
bench_last_n10,3716
bench_last_n100,3417
bench_last_n1000,3415
bench_len_n10,861
bench_len_n100,903
bench_len_n1000,916
bench_is_empty_n10,913
bench_is_empty_n100,902
bench_is_empty_n1000,902
bench_swap_n10,80669
bench_swap_n100,71902
bench_swap_n1000,71880
bench_swap_remove_n10,63998
bench_swap_remove_n100,63677
bench_swap_remove_n1000,54908
bench_remove_n10,178691
bench_remove_n100,1968550
bench_remove_n1000,19826811
bench_insert_n10,275300
bench_insert_n100,2048481
bench_insert_n1000,19908970
bench_reverse_n10,398092
bench_reverse_n100,3969213
bench_reverse_n1000,39680291
bench_fill_n10,381471
bench_fill_n100,3803543
bench_fill_n1000,38024243
bench_resize_grow_n10,507215
bench_resize_grow_n100,4929401
bench_resize_grow_n1000,49131101
bench_resize_shrink_n10,18128
bench_resize_shrink_n100,18117
bench_resize_shrink_n1000,18117
bench_store_vec_n10,370490
bench_store_vec_n100,3380719
bench_store_vec_n1000,33544199
bench_load_vec_n10,10617
bench_load_vec_n100,87707
bench_load_vec_n1000,860286
bench_iter_n10,28712
bench_iter_n100,278115
bench_iter_n1000,2770015
bench_clear_n10,17354
bench_clear_n100,17354
bench_clear_n1000,17354

--- Histogram ---

  push_n10                          │ █    72421
  push_n100                         │ █    63966
  push_n1000                        │ █    63944
  push_n_elems_into_empty_vec_n10   │ █   677638
  push_n_elems_into_empty_vec_n100  │ ██████  6718770
  push_n_elems_into_empty_vec_n1000 │ ████████████████████████████████████████████████████████████ 67150178
  pop_n10                           │ █    19817
  pop_n100                          │ █    19547
  pop_n1000                         │ █    19560
  get_n10                           │ █     3670
  get_n100                          │ █     3694
  get_n1000                         │ █     3397
  set_n10                           │ █    43518
  set_n100                          │ █    43571
  set_n1000                         │ █    35116
  first_n10                         │ █     3390
  first_n100                        │ █     3388
  first_n1000                       │ █     3377
  last_n10                          │ █     3716
  last_n100                         │ █     3417
  last_n1000                        │ █     3415
  len_n10                           │ █      861
  len_n100                          │ █      903
  len_n1000                         │ █      916
  is_empty_n10                      │ █      913
  is_empty_n100                     │ █      902
  is_empty_n1000                    │ █      902
  swap_n10                          │ █    80669
  swap_n100                         │ █    71902
  swap_n1000                        │ █    71880
  swap_remove_n10                   │ █    63998
  swap_remove_n100                  │ █    63677
  swap_remove_n1000                 │ █    54908
  remove_n10                        │ █   178691
  remove_n100                       │ █  1968550
  remove_n1000                      │ █████████████████ 19826811
  insert_n10                        │ █   275300
  insert_n100                       │ █  2048481
  insert_n1000                      │ █████████████████ 19908970
  reverse_n10                       │ █   398092
  reverse_n100                      │ ███  3969213
  reverse_n1000                     │ ███████████████████████████████████ 39680291
  fill_n10                          │ █   381471
  fill_n100                         │ ███  3803543
  fill_n1000                        │ █████████████████████████████████ 38024243
  resize_grow_n10                   │ █   507215
  resize_grow_n100                  │ ████  4929401
  resize_grow_n1000                 │ ███████████████████████████████████████████ 49131101
  resize_shrink_n10                 │ █    18128
  resize_shrink_n100                │ █    18117
  resize_shrink_n1000               │ █    18117
  store_vec_n10                     │ █   370490
  store_vec_n100                    │ ███  3380719
  store_vec_n1000                   │ █████████████████████████████ 33544199
  load_vec_n10                      │ █    10617
  load_vec_n100                     │ █    87707
  load_vec_n1000                    │ █   860286
  iter_n10                          │ █    28712
  iter_n100                         │ █   278115
  iter_n1000                        │ ██  2770015
  clear_n10                         │ █    17354
  clear_n100                        │ █    17354
  clear_n1000                       │ █    17354

═══════════════════════════════════════════════════════════════
  Running: storage_vec_s96
═══════════════════════════════════════════════════════════════

--- CSV (per-count baselines subtracted) ---
  empty-call baseline: 12274
  populate baselines : n10=654190  n100=6407538  n1000=63940946  
  store_vec baselines: n10=13928  n100=25725  n1000=142374  
test,gas
bench_push_n10,63959
bench_push_n100,63972
bench_push_n1000,63950
bench_push_n_elems_into_empty_vec_n10,642004
bench_push_n_elems_into_empty_vec_n100,6395352
bench_push_n_elems_into_empty_vec_n1000,63928760
bench_pop_n10,19505
bench_pop_n100,19547
bench_pop_n1000,19560
bench_get_n10,3360
bench_get_n100,3384
bench_get_n1000,3397
bench_set_n10,35056
bench_set_n100,35109
bench_set_n1000,35122
bench_first_n10,3391
bench_first_n100,3389
bench_first_n1000,3380
bench_last_n10,3404
bench_last_n100,3417
bench_last_n1000,3415
bench_len_n10,861
bench_len_n100,903
bench_len_n1000,916
bench_is_empty_n10,913
bench_is_empty_n100,902
bench_is_empty_n1000,902
bench_swap_n10,70483
bench_swap_n100,70496
bench_swap_n1000,70474
bench_swap_remove_n10,54912
bench_swap_remove_n100,54903
bench_swap_remove_n1000,54914
bench_remove_n10,160843
bench_remove_n100,1749344
bench_remove_n1000,17634805
bench_insert_n10,240528
bench_insert_n100,1829287
bench_insert_n1000,17716976
bench_reverse_n10,352834
bench_reverse_n100,3529395
bench_reverse_n1000,35294873
bench_fill_n10,337773
bench_fill_n100,3379325
bench_fill_n1000,33794825
bench_resize_grow_n10,477031
bench_resize_grow_n100,4607401
bench_resize_grow_n1000,45911101
bench_resize_shrink_n10,18128
bench_resize_shrink_n100,18117
bench_resize_shrink_n1000,18117
bench_store_vec_n10,394850
bench_store_vec_n100,3685419
bench_store_vec_n1000,36591199
bench_load_vec_n10,11241
bench_load_vec_n100,95512
bench_load_vec_n1000,938324
bench_iter_n10,27152
bench_iter_n100,262515
bench_iter_n1000,2614015
bench_clear_n10,17354
bench_clear_n100,17354
bench_clear_n1000,17354

--- Histogram ---

  push_n10                          │ █    63959
  push_n100                         │ █    63972
  push_n1000                        │ █    63950
  push_n_elems_into_empty_vec_n10   │ █   642004
  push_n_elems_into_empty_vec_n100  │ ██████  6395352
  push_n_elems_into_empty_vec_n1000 │ ████████████████████████████████████████████████████████████ 63928760
  pop_n10                           │ █    19505
  pop_n100                          │ █    19547
  pop_n1000                         │ █    19560
  get_n10                           │ █     3360
  get_n100                          │ █     3384
  get_n1000                         │ █     3397
  set_n10                           │ █    35056
  set_n100                          │ █    35109
  set_n1000                         │ █    35122
  first_n10                         │ █     3391
  first_n100                        │ █     3389
  first_n1000                       │ █     3380
  last_n10                          │ █     3404
  last_n100                         │ █     3417
  last_n1000                        │ █     3415
  len_n10                           │ █      861
  len_n100                          │ █      903
  len_n1000                         │ █      916
  is_empty_n10                      │ █      913
  is_empty_n100                     │ █      902
  is_empty_n1000                    │ █      902
  swap_n10                          │ █    70483
  swap_n100                         │ █    70496
  swap_n1000                        │ █    70474
  swap_remove_n10                   │ █    54912
  swap_remove_n100                  │ █    54903
  swap_remove_n1000                 │ █    54914
  remove_n10                        │ █   160843
  remove_n100                       │ █  1749344
  remove_n1000                      │ ████████████████ 17634805
  insert_n10                        │ █   240528
  insert_n100                       │ █  1829287
  insert_n1000                      │ ████████████████ 17716976
  reverse_n10                       │ █   352834
  reverse_n100                      │ ███  3529395
  reverse_n1000                     │ █████████████████████████████████ 35294873
  fill_n10                          │ █   337773
  fill_n100                         │ ███  3379325
  fill_n1000                        │ ███████████████████████████████ 33794825
  resize_grow_n10                   │ █   477031
  resize_grow_n100                  │ ████  4607401
  resize_grow_n1000                 │ ███████████████████████████████████████████ 45911101
  resize_shrink_n10                 │ █    18128
  resize_shrink_n100                │ █    18117
  resize_shrink_n1000               │ █    18117
  store_vec_n10                     │ █   394850
  store_vec_n100                    │ ███  3685419
  store_vec_n1000                   │ ██████████████████████████████████ 36591199
  load_vec_n10                      │ █    11241
  load_vec_n100                     │ █    95512
  load_vec_n1000                    │ █   938324
  iter_n10                          │ █    27152
  iter_n100                         │ █   262515
  iter_n1000                        │ ██  2614015
  clear_n10                         │ █    17354
  clear_n100                        │ █    17354
  clear_n1000                       │ █    17354
```
