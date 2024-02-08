# Keywords

The following list contains keywords that are reserved for current or
future use by the Sway language. As such, they cannot be used as
identifiers. Identifiers are names of functions, variables,
parameters, modules, constants, attributes, types or
traits, etc.

## Keywords Currently in Use

The following is a list of keywords currently in use, with their
functionality described.

- `as` - rename items in `use` statements, e.g., `use type::a as alias_name`
- [`abi`](../sway-program-types/smart_contracts.md#the-abi-declaration) - defines a smart contract ABI in a syntactically similar way to traits
- [`break`](../basics/control_flow.md#break-and-continue) - exit a loop immediately
- [`const`](../basics/constants.md) - define constant items
- [`continue`](../basics/control_flow.md#break-and-continue) - continue to the next loop iteration
- `else` - used in conjunction with `if` conditions for control flow constructs
- [`enum`](../basics/structs_tuples_and_enums.md#enums) - define an enumeration
- `false` - Boolean false literal
- [`fn`](../basics/functions.md)- define a function or the function pointer type
- [`if`](../basics/control_flow.md#if-expressions) - branch based on the result of a conditional expression
- `impl` - implement inherent or trait functionality
- `let` - bind a variable
- [`match`](../basics/control_flow.md#match-expressions) - exhaustively match a value to patterns
- `mod` - define a module
- `mut` - denote mutability in references, or pattern bindings
- `pub` - denote public visibility of Sway data structures, traits, or modules
- `ref` - bind by reference
- `return` - return early from a function
- `Self` - a type alias for the type we are defining or implementing
- `self` - method subject
- [`struct`](../basics/structs_tuples_and_enums.md#structs) - define a structure
- [`trait`](../advanced/traits.md#declaring-a-trait) - define a trait
- `true` - Boolean true literal
- [`type`](../advanced/advanced_types.md#creating-type-synonyms-with-type-aliases) - define a type alias or associated type
- `use` - bring symbols into scope
- `where` - specifies traits for generic types
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

## Special Keywords

### Program Keywords

Keywords associated with defining the type of Sway program to compile

- [`contract`](../sway-program-types/smart_contracts.md) - analogous to a deployed API with some database state
- [`library`](../sway-program-types/libraries.md) - Sway code that defines new common behavior
- [`predicate`](../sway-program-types/predicates.md) - programs that return a Boolean value and which represent ownership of some resource upon execution to true
- [`script`](../sway-program-types/scripts.md) - a runnable bytecode on the chain, which executes once to perform a task

### Attribute Keywords

Keywords associated with defining the functionality of attributes

- [`allow`](./attributes.md#allow) - overrides checks that would otherwise result in errors or warnings
- [`doc`](./attributes.md#doc) - specifies documentation
- [`inline`](./attributes.md#inline) - suggests that a copy of the attributed function should be placed in the caller, rather than generating code to call the function where it is defined
- [`payable`](./attributes.md#payable) - implies method is payable for compile time
- [`storage`](./attributes.md#storage) - declaration that contains a list of stored variables
- [`test`](./attributes.md#test) - marks a function to be executed as a test
- [`deprecated`](./attributes.md#deprecated) - marks an item as deprecated
