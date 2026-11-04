# Benchmark report

`xunilrj/branch-aware-memcpy-push-down` @`3657e330` vs `master` @`7dafb1c7`

## just pa diffs

_Skipped: need `just pa` CSVs for both master and `xunilrj/branch-aware-memcpy-push-down` in `/Users/xunilrj/github/FuelLabs/sway-memcpy-pushdown/test/perf_out`._

## fuel-o2-exports / order-book

|   | Improvements | Regressions |
| - | -: | -: |
| Count   | 33 | 2 |
| Average | 0.30% | -0.04% |
| Median  | 0.30% | -0.04% |
| Max     | 0.76% | -0.04% |
| Min     | 0.03% | -0.03% |

### Distribution

```
Histogram for Percentage (bars: 10, sum: 35n, max: 9n):

>= -0.04  <  0.04   |5n  14.29%|■■■■■■■■■■■■■■■■■■■■■■                 |
>= 0.04   <  0.12   |4n  11.43%|■■■■■■■■■■■■■■■■■■                     |
>= 0.12   <  0.2    |2n   5.71%|■■■■■■■■■                              |
>= 0.2    <  0.28   |4n  11.43%|■■■■■■■■■■■■■■■■■■                     |
>= 0.28   <  0.3600 |9n  25.71%|■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■|
>= 0.3600 <  0.4400 |6n  17.14%|■■■■■■■■■■■■■■■■■■■■■■■■■■             |
>= 0.4400 <  0.52   |2n   5.71%|■■■■■■■■■                              |
>= 0.52   <  0.6    |2n   5.71%|■■■■■■■■■                              |
>= 0.6    <  0.6799 |0n   0.00%|                                       |
>= 0.6799 <= 0.7599 |1n   2.86%|■■■■■                                  |
```

### Regressions

| Test | Before | After | Percentage |
| ---- | -----: | ----: | ---------: |
| "single_level/total_gas" | 4113819 | 4115124 | -0.03% |
| "single_level/gas_per_match" | 8227 | 8230 | -0.04% |


## fuel-o2-exports / trade-account

|   | Improvements | Regressions |
| - | -: | -: |
| Count   | 21 | — |
| Average | 0.21% | — |
| Median  | 0.19% | — |
| Max     | 0.41% | — |
| Min     | 0.01% | — |

### Distribution

```
Histogram for Percentage (bars: 10, sum: 21n, max: 6n):

>= 0.01   <  0.0499 |1n   4.76%|■■■■■■■                                |
>= 0.0499 <  0.09   |2n   9.52%|■■■■■■■■■■■■■                          |
>= 0.09   <  0.13   |2n   9.52%|■■■■■■■■■■■■■                          |
>= 0.13   <  0.1699 |1n   4.76%|■■■■■■■                                |
>= 0.1699 <  0.2099 |6n  28.57%|■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■|
>= 0.2099 <  0.2499 |1n   4.76%|■■■■■■■                                |
>= 0.2499 <  0.2899 |2n   9.52%|■■■■■■■■■■■■■                          |
>= 0.2899 <  0.3299 |1n   4.76%|■■■■■■■                                |
>= 0.3299 <  0.3699 |3n  14.29%|■■■■■■■■■■■■■■■■■■■■                   |
>= 0.3699 <= 0.4099 |2n   9.52%|■■■■■■■■■■■■■                          |
```
