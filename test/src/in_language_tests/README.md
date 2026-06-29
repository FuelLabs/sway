# In-language tests for the Sway standard library

These are in-language unit tests  that test the Sway standard library (`sway-lib-std`).
Each test project lives under [`test_programs/`](./test_programs) and is a member of
the workspace [`Forc.toml`](./Forc.toml).

## Running the tests

Use [`run_in_language_tests.sh`](./run_in_language_tests.sh):

```sh
# Run all test projects (parallel by default; concise per-project results, fastest
# for "do all tests pass?").
./run_in_language_tests.sh

# Run only the projects matching a regex (matches project dir names and *.sw contents).
./run_in_language_tests.sh --filter '^alloc_$'

# Run sequentially, printing the full `forc test` output as it runs.
./run_in_language_tests.sh --sequential
```

Extra arguments are forwarded to `forc test` (e.g. `--release`, `--experimental ...`).
See the header of the script for the full set of options and run modes.

## Conventions

### Project structure mirrors the std library module structure

The structure of the test projects reflects the structure of the standard
library modules. Ideally there is **one test project per top-level std module**,
named after that module (e.g. the `bytes` project tests `std::bytes`).

When a std module has submodules, the test project mirrors that structure using
Sway submodules: one `src/<submodule>.sw` per std submodule, declared from the
entry file. For example, `std::array_conversions` (with its `b256`, `u16`, `u32`,
`u64`, `u256` submodules) is tested by the single `array_conversions` project.

Some modules still need more than one project — for example,
`std::storage::storage_vec`, whose tests are split across many projects
(`storage_vec_insert`, `storage_vec_remove`, ...) because a single project would
be too large or slow to compile. Such related projects are grouped under a plain
folder — neither a project nor a workspace — to keep `test_programs/` tidy and
easier to navigate.

Shared test helpers that are *not* themselves test projects live outside
`test_programs/`, alongside it. E.g., the [`test_types`](./test_types) library.

### Entry file is named after the project, not `main.sw`

Each project's entry source file is named after the project rather than
`main.sw` (e.g. the `string` project's entry is `src/string.sw`,
`storage_vec_insert`'s is `src/storage_vec_insert.sw`). For a project that
corresponds 1:1 to a std module, the project — and hence its entry file — is
named after that module. This improves discoverability: searching for
`string.sw` in an IDE surfaces both the std module and its test alongside each
other.

For a [reserved project name](../../../forc-util/src/restricted.rs) the entry
file uses the bare module name, even though the project name is postfixed (e.g.
the `alloc_` project's entry is `src/alloc.sw`).

### Execution context (`contract` vs non-contract)

Some std functionality behaves differently depending on the program kind a test
runs in (`library`/`script` vs `contract`) — e.g. `AssetId::default()` and
`ContractId::this()` only return meaningful values within a contract.

For the vast majority of tests the context is irrelevant, so they all live
together in the canonical `<module>` project. When a module needs tests that
require a contract (`impl Contract` + `abi(...)` calls), that `<module>` project
is simply a `contract;` program and hosts both those contract tests and all the
context-independent inline tests for the module.

The exception is a test that **must not** run in a given context. Such a test is
split out into a separate `<module>_no_<context>` project. For example,
`asset_id_no_contract` holds the single test that exercises `AssetId::default()`
*outside* a contract (where it intentionally returns erroneous data); every other
`asset_id` test lives in the `asset_id` contract project.

### Contract tests use `impl Contract`, not an explicit `abi`

Contract tests do **not** declare an explicit `abi` unless one is strictly
needed, but rather use `impl Contract`.

### Reserved project names get a trailing `_`

If a module name is a [reserved Forc project name](../../../forc-util/src/restricted.rs)
(e.g. `alloc`), the test project name is postfixed with an underscore (e.g.
`alloc_`) so that the project name still reflects the module it tests.

### Prefer `assert_eq` / `assert_ne` over `assert(a == b)`

Use `assert_eq(a, b)` and `assert_ne(a, b)` instead of `assert(a == b)` and
`assert(a != b)` — the failure messages report both operands.
