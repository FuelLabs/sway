# `perf_out` Folder

The `perf_out` folder contains temporary files with performance data collected from tests. E.g, `*-gas-usages-*.csv` or `*-bytecode-sizes-*.cvs`.

The performance data are .gitignored. To delete them locally after an analysis is done, call `just pr`. `pr` is an alias for `perf-remove` `just` recipe that removes all temporary files related to collected performance data.
