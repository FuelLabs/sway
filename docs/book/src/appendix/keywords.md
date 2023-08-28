# Appendix A: Keywords

The following list contains keywords that are reserved for current or
future use by the Sway language. As such, they cannot be used as
identifiers. Identifiers are names of functions, variables,
parameters, modules, constants, attributes, types or
traits, ect.

## Keywords Currently in Use

The following is a list of keywords currently in use, with their
functionality described.

- `as` - rename items in `use` statements, eg `use type::a as alias_name`
- [`abi`](/docs/book/src/sway-program-types/smart_contracts.md) - defines a smart contract ABI in a syntactcally similar way to traits
- [`break`](/docs/book/src/basics/control_flow.md) - exit a loop immediately
- [`const`](/docs/book/src/basics/constants.md) - define constant items
- [`continue`](/docs/book/src/basics/control_flow.md) - continue to the next loop iteration
- `else` - used in conjunction with `if` conditions for control flow constructs
- [`enum`](/docs/book/src/basics/structs_tuples_and_enums.md) - define an enumeration
- `false` - Boolean false literal
- [`fn`](/docs/book/src/basics/functions.md)- define a function or the function pointer type
- [`if`](/docs/book/src/basics/control_flow.md) - branch based on the result of a conditional expression
- `impl` - implement inherent or trait functionality
- `let` - bind a variable
- [`match`](/docs/book/src/basics/control_flow.md) - exhaustfully match a value to patterns
- `mod` - define a module
- `mut` - denote mutability in references, or pattern bindings
- `pub` - denote public visibility of Sway data structures, traits, or modules
- `ref` - bind by reference
- `return` - return early from a function
- `Self` - a type alias for the type we are defining or implementing
- `self` - method subject
- [`struct`](/docs/book/src/basics/structs_tuples_and_enums.md) - define a structure
- [`trait`](/docs/book/src/advanced/traits.md) - define a trait
- `true` - Boolean true literal
- [`type`](/docs/book/src/advanced/generic_types.md) - define a type alias or associated type
- `use` - bring symbols into scope
- `where` - specifies traits for generic types
- [`while`](/docs/book/src/basics/control_flow.md) - loop conditionally based on the result of an expression

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

- [`contract`](/docs/book/src/sway-program-types/smart_contracts.md) - analogous to a deployed API with some database state
- [`library`](/docs/book/src/sway-program-types/libraries.md) - Sway code that defines new common behavior
- [`predicate`](/docs/book/src/sway-program-types/predicates.md) - programs that return a Boolean value and which represent ownership of some resource upon execution to true
- [`script`](/docs/book/src/sway-program-types/scripts.md) - a runnable bytecode on the chain, which executes once to preform a task

### Attribute Keywords

Keywords associated with defining the funcitonallity of attributes

- [`allow`](/docs/book/src/reference/attributes.md) - overrides checks that would otherwise result in errors or warnings
- [`doc`](/docs/book/src/reference/attributes.md) - specifies documentation
- [`inline`](/docs/book/src/reference/attributes.md) - suggests that a copy of the attributed function should be placed in the caller, rather than generating code to call the function where it is defined
- [`payable`](/docs/book/src/reference/attributes.md) - implies method is payable for compile time
- [`storage`](/docs/book/src/reference/attributes.md) - declaration that contains a list of stored variables
- [`test`](/docs/book/src/reference/attributes.md) - marks a function to be executed as a test
