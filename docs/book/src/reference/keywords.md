# Keywords

The following list contains keywords that are reserved for current or
future use by the Sway language. As such, **they cannot be used as
identifiers**. Identifiers are names of functions, variables,
parameters, modules, constants, attributes, types or
traits, etc.

## Keywords Currently in Use

The following is an alphabetically sorted list of keywords currently in use, with their
functionality shortly described.

- `as` - rename items in `use` statements, e.g., `use type::type_name as alias_name;`
- [`abi`](../sway-program-types/smart_contracts.md#the-abi-declaration) - define a smart contract ABI in a syntactically similar way to traits
- [`asm`](../advanced/assembly.md) - define an assembly block
- [`break`](../basics/control_flow.md#break-and-continue) - exit a loop immediately
- `configurable` - define configurable constants
- [`const`](../basics/constants.md) - define constant
- [`continue`](../basics/control_flow.md#break-and-continue) - continue to the next loop iteration
- [`contract`](../sway-program-types/smart_contracts.md) - define contract program type
- `else` - used in conjunction with `if` conditions for control flow constructs
- [`enum`](../basics/structs_tuples_and_enums.md#enums) - define an enum
- `false` - Boolean false literal
- [`for`](../basics/control_flow.md#for) - loop based on iterators
- [`fn`](../basics/functions.md)- define a function
- [`if`](../basics/control_flow.md#if-expressions) - branch based on the result of a conditional expression
- `impl` - implement inherent or trait functionality
- `let` - bind a variable
- [`library`](../sway-program-types/libraries.md) - define library program type
- [`match`](../basics/control_flow.md#match-expressions) - exhaustively match a value to patterns
- `mod` - define a module
- `mut` - denote mutability
- `pub` - denote public visibility
- [`predicate`](../sway-program-types/predicates.md) - define predicate program type
- `ref` - bind by reference
- `return` - return early from a function
- [`script`](../sway-program-types/scripts.md) - define script program type
- `Self` - a type alias for the type we are defining or implementing
- `self` - method call target
- [`storage`](../blockchain-development/storage.md) - define a storage declaration
- `str`- string slice
- [`struct`](../basics/structs_tuples_and_enums.md#structs) - define a structure
- [`trait`](../advanced/traits.md#declaring-a-trait) - define a trait
- `true` - Boolean true literal
- [`type`](../advanced/advanced_types.md#creating-type-synonyms-with-type-aliases) - define a type alias or associated type
- `use` - bring symbols into scope
- `where` - specifies trait constraints for generic type arguments
- [`while`](../basics/control_flow.md#while) - loop conditionally based on the result of an expression

## Keywords Reserved for Possible Future Use

- `abstract`
- `async`
- `await`
- `become`
- `box`
- `do`
- `dyn`
- `extern`
- `for`
- `in`
- `loop`
- `macro`
- `move`
- `override`
- `priv`
- `static`
- `super`
- `try`
- `typeof`
- `unsafe`
- `unsized`
- `virtual`
- `yield`
