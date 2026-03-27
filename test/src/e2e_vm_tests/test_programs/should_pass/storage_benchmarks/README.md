# Storage Benchmarks

Benchmarks for measuring the performance (gas cost) of storage access operations.

## Benchmark projects

| Project | Description |
|---|---|
| `storage_fields` | Read, write, and clear operations on full storage fields of various types and sizes. |
| `storage_fields_partial_access` | Read and write operations on nested sub-fields of storage structs (partial access). |

Note that `storage_fields` costs are also the costs of inserting, reading, and clearing values in a `StorageMap`, reduced for the cost of key calculation.

## Shared library

The `stored_types` library defines the struct types used across all benchmark projects. Structs range from 24 bytes (`Struct24`) to 552 bytes (`Struct552`) and are composed hierarchically from smaller structs and `u64` fields.

The struct sizes to use in benchmarks are taken from real-life Sway projects.

## Running the benchmarks

### Prerequisites

Build the Sway toolchain in release mode (from the repo root):

```bash
cargo build -r -p forc
```

### Run all benchmarks

```bash
./bench.sh
```

### Run a single benchmark project

```bash
./bench.sh storage_fields
./bench.sh storage_fields_partial_access
```

### Run a benchmark project directly with `forc`

From the repo root:

```bash
cargo r -r -p forc -- test --release -p test/src/e2e_vm_tests/test_programs/should_pass/storage_benchmarks/storage_fields/
```

## Output

The `bench.sh` script produces two output formats for each project:

### CSV

Comma-separated values with columns `test,gas` where gas is the cost with the baseline (cost of an empty contract method call) subtracted:

```
test,gas
bench_bool_read,854
bench_bool_write,16845
...
```

### Histogram

A visual bar chart printed to the console, showing relative gas costs:

```
  bench_bool_read  │ █ 854
  bench_bool_write │ █████ 16845
  ...
```
