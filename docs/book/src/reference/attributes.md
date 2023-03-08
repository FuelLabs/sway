# Attributes

The Sway compiler supports a list of attributes that perform various operations that are useful for building, testing and documenting Sway programs. Below is a list of all available attributes:

## Allow

The `#[allow(dead_code)]` attribute overrides the check for dead code so that violations will go unreported.

## Doc

The `#[doc(..)]` attribute specifies documentation.

Line doc comments beginning with exactly three slashes `///`, are interpreted as a special syntax for doc attributes. That is, they are equivalent to writing `#[doc("...")]` around the body of the comment, i.e., `/// Foo` turns into `#[doc("Foo")]`

Line comments beginning with `//!` are doc comments that apply to the module of the source file they are in. That is, they are equivalent to writing `#![doc("...")]` around the body of the comment. `//!` module level doc comments should be at the top of Sway files.

Documentation can be generated from doc attributes using `forc doc`.

## Inline

The inline attribute suggests that a copy of the attributed function should be placed in the caller, rather than generating code to call the function where it is defined.

> **Note**: The Sway compiler automatically inlines functions based on internal heuristics. Incorrectly inlining functions can make the program slower, so this attribute should be used with care.

The `#[inline(never)]` attribute *suggests* that an inline expansion should never be performed.

The `#[inline(always)]` attribute *suggests* that an inline expansion should always be performed.

> **Note**: `#[inline(..)]` in every form is a hint, with no *requirements*
 on the language to place a copy of the attributed function in the caller.

## Payable

The lack of `#[payable]` implies the method is non-payable. When calling an ABI method that is non-payable, the compiler emits an error if the amount of coins forwarded with the call is not guaranteed to be zero. Note that this is strictly a compile-time check and does not incur any runtime cost.

## Storage

In Sway, functions are pure by default but can be opted into impurity via the `storage` function attribute. The `storage` attribute may take `read` and/or `write` arguments indicating which type of access the function requires.

The `#[storage(read)]` attribute indicates that a function requires read access to the storage.

The `#[storage(write)]` attribute indicates that a function requires write access to the storage.

More details in [Purity](../blockchain-development/purity.md).

## Test

The `#[test]` attribute marks a function to be executed as a test.

The `#[test(should_revert)]` attribute marks a function to be executed as a test that should revert.

More details in [Unit Testing](../testing/unit-testing.md).
