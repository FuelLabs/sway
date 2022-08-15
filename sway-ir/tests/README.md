# Notes on the Inliner Unit Testing

Each of the files in the `inline` directory are passed through the inliner and verified using
`FileCheck`.

## Parameters

The first line of the IR file must be a comment containing the parameters for the pass.  These may
be:

* The single word `all`, indicating all `CALL`s found throughout the input will be inlined.
* A combination of sizes which are passed to the `optimize::inline::is_small_fn()` function:
  * `blocks N` to indicate a maximum of `N` allowed blocks constraint.
  * `instrs N`  to indicate a maximum of `N` allowed instructions constraint.
  * `stack N` to indicate a maximum of `N` for stack size constraint.

Any keyword found later in the line will override an earlier parameter.  `all` will override any
other constraint.

### Example

To just inline everything:

```rust
// all
```

To inline only functions which have at most 2 blocks:

```rust
// blocks 2
```

To inline only functions which have at most 2 blocks, at most 20 instructions and no more than 10
stack elements:

```rust
// blocks 2 instrs 20 stack 10
```

See the source for `optimize::inline::is_small_fn()` for further clarification.

### Caveats

This is a little bit lame and perhaps a proper looking command line (and parser) would be better,
e.g., `// run --blocks 2 --instrs 20` but this will do for a start.
