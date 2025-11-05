# Attributes

Attributes are a form of metadata that can additionally instruct Sway compiler or other tools like `forc test`. Attributes can annotate different language elements, like, e.g., items, enum variants, struct fields, etc.

Below is the list of attributes supported by the Sway compiler, ordered alphabetically:

- [ABI Name](#abi-name)
- [Allow](#allow)
- [Cfg](#cfg)
- [Deprecated](#deprecated)
- [Error](#error)
- [Error Type](#error-type)
- [Event](#event--indexed)
- [Fallback](#fallback)
- [Indexed](#event--indexed)
- [Inline](#inline)
- [Payable](#payable)
- [Storage](#storage)
- [Test](#test)
- [Tracing](#tracing)

## ABI Name

The `#[abi_name]` attribute allows to specify the ABI name for an item.
This means that when an ABI JSON file is generated, the name that is output is the one specified
by the attribute. This can be useful to allow renaming items, while allowing for keeping backwards
compatibility at the contract ABI level.

> **Note**: At the moment, only enum and struct types support the attribute.

In the example that follows, we originally had `MyStruct` and `MyEnum` types, which we, later on, renamed to `RenamedMyStruct` and `RenamedMyEnum` in code. To keep the backward compatibility of the ABI, we annotate the types with the `#[abi_name]` attribute and give them the original names:

```sway
contract;

#[abi_name(name = "MyStruct")]
struct RenamedMyStruct {}

#[abi_name(name = "MyEnum")]
enum RenamedMyEnum {
  A: ()
}

abi MyAbi {
    fn my_struct() -> RenamedMyStruct;
    fn my_enum() -> RenamedMyEnum;
}

impl MyAbi for Contract {
  fn my_struct() -> RenamedMyStruct { RenamedMyStruct{} }
  fn my_enum() -> RenamedMyEnum { RenamedMyEnum::A }
}
```

This generates the following JSON ABI:

```json
{
  "concreteTypes": [
    {
      "concreteTypeId": "215af2bca9e1aa8fec647dab22a0cd36c63ce5ed051a132d51323807e28c0d67",
      "metadataTypeId": 1,
      "type": "enum MyEnum"
    },
    {
      "concreteTypeId": "d31db280ac133d726851d8003bd2f06ec2d3fc76a46f1007d13914088fbd0791",
      "type": "struct MyStruct"
    }
  ],
  ...
}
```

We get the same JSON ABI output both before and after renaming the types, due to attributing them with
`#[abi_name(name = ...)]`, which forces them to be generated with their previous Sway names.
This means consumers of this contract will still get the original names, keeping compatibility at the ABI level.

## Allow

The `#[allow(...)]` attribute disables compiler checks so that certain warnings will go unreported. The following warnings can be disabled:

- `#[allow(dead_code)]` disables warnings for dead code;
- `#[allow(deprecated)]` disables warnings for usage of deprecated elements, like, e.g., structs, functions, enum variants, etc.

## Cfg

The `#[cfg(...)]` attribute allows conditional compilation. The annotated code element will be compiled only if the condition in to the `cfg` attribute evaluates to true. The following conditions can be expressed:

- `#[cfg(target = "<target>")]` where `<target>` can be either "evm" or "fuel";
- `#[cfg(program_type = "<program_type>")]` where `<program_type>` can be either "predicate", "script", "contract", or "library";
- `#[cfg(experimental_<feature_flag> = true/false)]` where `<feature_flag>` is one of the known [experimental feature](../reference/experimental_features.md) flags.

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

## Error Type

The `#[error_type]` marks an enum as error type enum:

```sway
#[error_type]
enum SomeErrors {
    ...
}
```

All variants of an error type enum must be annotated with the [`#[error]` attribute](#error). Error type enums are meant to be use in `panic` expressions for rich error reporting.

## Event / Indexed

The `#[event]` attribute marks a struct or enum as an event that can be emitted by a contract.

The `#[indexed]` attribute can be applied to fields within structs that are attributed with `#[event]`. This is particularly useful for event structs, allowing for efficient filtering and searching of emitted events based on the values of these fields.

When using this attribute, the indexed fields must be applied sequentially to the initial set of fields in a struct.

This attribute can only be applied to fields whose type is an exact size ABI type. The exact size ABI types include:

- `bool`
- `u8`, `u16`, `u32`, `u64`, `u256`
- `numeric`
- `b256`
- `Address`
- `str[N]`
- Tuples containing only exact size types
- Structs containing only exact size types
- Arrays of exact size types with a literal length
- Type aliases to exact size types

Additionally it causes the event types to be included in the JSON ABI representation for the contract.

```sway
#[event]
struct MyEventStruct {
    #[indexed]
    id: u64,
    sender: Identity,
}

#[event]
enum MyEventEnum {
    A: (),
    B: (),
}
```

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

## Tracing

The tracing attribute tells the compiler if a function should be included in a backtrace of a revert caused by a `panic` expression call.

The `#[tracing(never)]` signals the compiler not to include the function in a backtrace, unless the `backtrace` build option is set to `all`.

The `#[tracing(always)]` signals the compiler to always include the function in a backtrace, unless the `backtrace` build option is set to `none`.

More details in [Irrecoverable Errors](../basics/error_handling.md#irrecoverable-errors).
