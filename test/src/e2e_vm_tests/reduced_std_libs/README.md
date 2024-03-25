# Reduced Versions of the Sway Standard Library

This folder contains reduced versions of the Sway Standard Library meant to be used in tests that only need a limited `std` functionality. Until we get incremental compilation or reuse of a once-compiled `std` across tests, we want to minimize dependencies on `std`. The reason for this is the significant increase in test compilation time when `std` gets compiled as a test dependency.

In general, when writing a test try to avoid depending on `std` unless the test actually needs `std` functionality.

If the test depends only on a narrow subset of `std`, try first to include one of the following reduced versions of `std` as dependency. The compilation time of reduced versions is negligible compared to the compile time of the whole `std`.

If none of the reduced versions contain the modules needed by the test, include the whole `std` from the `sway-lib-std`.

## Content of the Reduced Versions

Each reduced version brings a small additional functionality on top of the previous one. The versions are listed below, ordered by increasing functionality.
 
### `assert` (in `sway-lib-std-assert`)
Contains:
- asserting
- logging
- reverting
 
### `option-result` (in `sway-lib-std-option-result`)
Contains:
- everything available in `assert`
- `Option`
- `Result`
 
### `vec` (in `sway-lib-std-vec`)
Contains:
- everything available in `option-result`
- `Vec`
- `Iterator` trait
- `From` and `Into` traits

### `conversions` (in `sway-lib-std-conversions`)
Contains:
- everything available in `vec`
- intrinsics
- `Bytes`
- bytes conversions
- array conversions
- primitive conversions
