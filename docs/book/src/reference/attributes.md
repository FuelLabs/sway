# Attributes

The Sway compiler supports a list of attributes that perform various operations that are useful for building, testing and documenting Sway programs. Below is a list of all available attributes:

## Allow

The `#[allow(...)]` attribute overrides checks so that violations will go unreported. The following checks can be disabled:

- `#[allow(dead_code)]` disable checks for dead code;
- `#[allow(deprecated)]` disables checks for usage of deprecated structs, functions and other items.

## Doc

- `#[doc(..)]` or /// are used for documentation comments
- `///` Foo is equivalent to `#[doc("Foo")]`
- `//!` is used for module-level documentation comments
- `//!` module-level doc comments should be at the top of Sway files
- Documentation can be generated using `forc doc`

## Inline

-  The inline attribute suggests that a copy of the attributed function should be placed in the caller, rather than generating code to call the function where it is defined.
- The Sway compiler automatically inlines functions based on internal heuristics.
- Incorrectly inlining functions can make the program slower, so this attribute should be used with care.
- `#[inline(never)]` attribute suggests that an inline expansion should never be performed.
- `#[inline(always)]` attribute suggests that an inline expansion should always be performed.
- `#[inline(..)]` in every form is a hint, with no requirements on the language to place a copy of the attributed function in the caller.

## Payable

- Lack of `#[payable]` implies the method is non-payable
- Compiler emits an error if a non-payable method is called with a non-zero amount
- This is a compile-time check and does not incur any runtime cost

## Storage

- Functions are pure by default
- `#[storage(read)]` indicates the function requires read access to storage
- `#[storage(write)]` indicates the function requires write access to storage
- Impurity can be opted into using the `storage` function attribute
- `storage` attribute may take `read `and/or `write` arguments

## Test

- `#[test]` marks a function as a test case
- `#[test(should_revert)]` marks a test case that should revert

More details in [Unit Testing](../testing/unit-testing.md).

## Deprecated

 - `#[deprecated]` marks an item as deprecated
Compiler emits a warning for every usage of the deprecated item
- Warning can be disabled using `#[allow(deprecated)]`
- Custom deprecation message can be provided with `#[deprecated(note = "...")]`

## Fallback

- `#[fallback]` marks a function as the contract's fallback function
- The fallback function is called when a contract is called, and the contract selection fails
