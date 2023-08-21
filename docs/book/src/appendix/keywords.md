## [Appendix A: Keywords](#appendix-a:-keywords)

The following list contains keywords that are reserved for current or
future use by the Sway language. As such, they cannot be used as
identifiers. Identifiers are names of functions, variables,
parameters, modules, constants, attributes, types or
traits, ect.

### [Keywords Currently in Use](#keywords-currently-in-use)

The following is a list of keywords currently in use, with their
functionality described.

- `as` - perform primitive casting, or rename items in `use` statements
- [`abi`](https://fuellabs.github.io/sway/master/book/sway-program-types/smart_contracts.html#the-abi-declaration) - defines a smart contract ABI in a syntactcally similar way to traits
- [`break`](https://fuellabs.github.io/sway/v0.44.0/book/basics/control_flow.html#break-and-continue) - exit a loop immediately
- [`const`](https://fuellabs.github.io/sway/v0.44.0/book/basics/constants.html) - define constant items
- [`continue`](https://fuellabs.github.io/sway/v0.44.0/book/basics/control_flow.html#break-and-continue) - continue to the next loop iteration
- `dep` - call dependencies outside of the current forc.toml scope
- `else` - used in conjunction with `if` conditions for control flow constructs
- [`enum`](https://fuellabs.github.io/sway/v0.44.0/book/basics/structs_tuples_and_enums.html#enums) - define an enumeration
- `false` - Boolean false literal
- [`fn`](https://fuellabs.github.io/sway/master/book/basics/functions.html)- define a function or the function pointer type
- [`if`](https://fuellabs.github.io/sway/v0.44.0/book/basics/control_flow.html#if-expressions) - branch based on the result of a conditional expression
- `impl` - implement inherent or trait functionality
- `let` - bind a variable
- [`match`](https://fuellabs.github.io/sway/v0.44.0/book/basics/control_flow.html#match-expressions) - exhaustfully match a value to patterns
- `mod` - define a module
- `mut` - denote mutability in references, or pattern bindings
- `pub` - denote public visibility of Sway data structures, traits, or modules
- `ref` - bind by reference
- `return` - return early from a function
- `Self` - a type alias for the type we are defining or implementing
- `self` - method subject
- [`struct`](https://fuellabs.github.io/sway/v0.44.0/book/basics/structs_tuples_and_enums.html#structs) - define a structure
- [`trait`](https://fuellabs.github.io/sway/master/book/advanced/traits.html#declaring-a-trait) - define a trait
- `true` - Boolean true literal
- `use` - bring symbols into scope
- `where` - specifies traits for generic types
- [`while`](https://fuellabs.github.io/sway/v0.44.0/book/basics/control_flow.html#while) - loop conditionally based on the result of an expression

### [Keywords Reserved for Possible Future Use](#keywords-reserved-for-possible-future-use)

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
- `type`
- `typeof`
- `unsafe`
- `unsized`
- `virtual`
- `yield`

## [Special Keywords](#special-keywords)

### [Program Keywords](#program-keywords)

Keywords associated with defining the type of Sway program to compile

- [`contract`](https://fuellabs.github.io/sway/master/book/sway-program-types/smart_contracts.html) - analogous to a deployed API with some database state
- [`library`](https://fuellabs.github.io/sway/master/book/sway-program-types/libraries.html) - Sway code that defines new common behavior 
- [`predicate`](https://fuellabs.github.io/sway/master/book/sway-program-types/predicates.html) - programs that return a Boolean value and which represent ownership of some resource upon execution to true
- [`script`](https://fuellabs.github.io/sway/master/book/sway-program-types/scripts.html) - a runnable bytecode on the chain, which executes once to preform a task


### [Attribute Keywords](#attribute-keywords)

Keywords associated with defining the funcitonallity of attributes

- [`allow`](https://fuellabs.github.io/sway/master/book/reference/attributes.html#allow) - overrides checks that would otherwise result in errors or warnings
- [`doc`](https://fuellabs.github.io/sway/master/book/reference/attributes.html#doc) - specifies documentation
- [`inline`](https://fuellabs.github.io/sway/master/book/reference/attributes.html#inline) - suggests that a copy of the attributed function should be placed in the caller, rather than generating code to call the function where it is defined
- [`payable`](https://fuellabs.github.io/sway/master/book/reference/attributes.html#payable) - implies method is payable for complie time
- [`storage`](https://fuellabs.github.io/sway/master/book/reference/attributes.html#storage) - declaration that contains a list of stored variables
- [`test`](https://fuellabs.github.io/sway/master/book/reference/attributes.html#test) - marks a function to be executed as a test