# Attributes

Attributes are a form of metadata that can additionally instruct Sway compiler or other tools like `forc test`. Attributes can annotate different language elements, like, e.g., items, enum variants, struct fields, etc.

Below is the list of attributes supported by the Sway compiler, ordered alphabetically:

- [Allow](#allow)
- [Cfg](#cfg)
- [Deprecated](#deprecated)
- [Error](#error)
- [Error Type](#error-type)
- [Fallback](#fallback)
- [Inline](#inline)
- [Payable](#payable)
- [Storage](#payable)
- [Test](#test)

## Allow

The `#[allow(...)]` attribute disables compiler checks so that certain warnings will go unreported. The following warnings can be disabled:

- `#[allow(dead_code)]` disables warnings for dead code;
- `#[allow(deprecated)]` disables warnings for usage of deprecated elements, like, e.g., structs, functions, enum variants, etc.

## Cfg

The `#[cfg(...)]` attribute allows conditional compilation. The annotated code element will be compiled only if the condition in to the `cfg` attribute evaluates to true. The following conditions can be expressed:

- `#[cfg(target = "<target>")]` where `<target>` can be either "evm" or "fuel";
- `#[cfg(program_type = "<program_type>")]` where `<program_type>` can be either "predicate", "script", "contract", or "library";
- `#[cfg(experimental_<feature_flag> = true/false)]` where `<feature_flag>` is one of the known experimental feature flags.

## Deprecated

The `#[deprecated]` attribute marks an item as deprecated and makes the compiler emit a warning for every usage of the deprecated item. This warning can be disabled using `#[allow(deprecated)]`.

It is possible to improve the warning message with `#[deprecated(note = "Your deprecation message.")]`

## Error

The `#[error]` defines an error message for an error type enum variant:

```sway
#[error_type]
enum SomeErrors {
    #[error(m = "An unexpected error occurred.")]
    UnexpectedError: (),
}
```

> **Note**: Error types are still an experimental feature. For more info, see the [tracking issue for "Error types"](https://github.com/FuelLabs/sway/issues/6765).

## Error Type

The `#[error_type]` marks an enum as error type enum:

```sway
#[error_type]
enum SomeErrors {
    ...
}
```

All variants of an error type enum must be annotated with the [`#[error]` attribute](#error). Error type enums are meant to be use in `panic` expressions for rich error reporting.

> **Note**: Error types are still an experimental feature. For more info, see the [tracking issue for "Error types"](https://github.com/FuelLabs/sway/issues/6765).

## Fallback

The `#[fallback]` attribute makes the compiler use the marked function as the contract call fallback function. This means that, when a contract method is called, and the contract method selection fails, the fallback function will be called instead.

More details in [Calling Contracts](../blockchain-development/calling_contracts.md#fallback).

## Inline

The inline attribute *suggests* to the compiler if a copy of the annotated function should be placed in the caller, rather than generating code to call the function where it is defined.

The `#[inline(never)]` attribute *suggests* that an inline expansion should never be performed.

The `#[inline(always)]` attribute *suggests* that an inline expansion should always be performed.

> **Note**: `#[inline(..)]` in every form is a hint, with no *requirements* on the compiler to place a copy of the annotated function in the caller. The Sway compiler automatically inlines functions based on internal heuristics. Incorrectly inlining functions can make the program slower, so this attribute should be used with care.

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
