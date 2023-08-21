## [Appendix A: Keywords](#appendix-a:-keywords)

The following list contains keywords that are reserved for current or
future use by the Sway language. As such, they cannot be used as
identifiers. Identifiers are names of functions, variables,
parameters, struct fields, modules, constants, attributes, types, or
traits.

### [Keywords Currently in Use](#keywords-currently-in-use)

The following is a list of keywords currently in use, with their
functionality described.

- `as` - perform primitive casting, disambiguate the specific trait
containing an item, or rename items in `use` statements
- [`abi`](https://fuellabs.github.io/sway/master/book/sway-program-types/smart_contracts.html#the-abi-declaration) - defines a smart contract ABI in a syntactcally similar way to traits
- [`break`](https://fuellabs.github.io/sway/v0.44.0/book/basics/control_flow.html#break-and-continue) - exit a loop immediately
- [`const`](https://fuellabs.github.io/sway/v0.44.0/book/basics/constants.html) - define constant items or constant raw pointers
- [`continue`](https://fuellabs.github.io/sway/v0.44.0/book/basics/control_flow.html#break-and-continue) - continue to the next loop iteration
- `dep` - call dependances outside of .toml
- `else` - fallback for `if` and `if let` control flow constructs
- [`enum`](https://fuellabs.github.io/sway/v0.44.0/book/basics/structs_tuples_and_enums.html#enums) - define an enumeration
- `false` - Boolean false literal
- [`fn`](https://fuellabs.github.io/sway/master/book/basics/functions.html)- define a function or the function pointer type
- [`if`](https://fuellabs.github.io/sway/v0.44.0/book/basics/control_flow.html#if-expressions) - branch based on the result of a conditional expression
- `impl` - implement inherent or trait functionality
- `let` - bind a variable
- [`match`](https://fuellabs.github.io/sway/v0.44.0/book/basics/control_flow.html#match-expressions) - match a value to patterns
- `mod` - define a module
- `mut` - denote mutability in references, raw pointers, or pattern bindings
- `pub` - denote public visibility in struct fields, impl blocks, or modules
- `ref` - bind by reference
- `return` - return from fuction
- `Self` - a type alias for the type we are defining or implementing
- `self` - method subject or current module
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

- [`contract`](https://fuellabs.github.io/sway/master/book/sway-program-types/smart_contracts.html) - calls smart contract
- [`library`](https://fuellabs.github.io/sway/master/book/sway-program-types/libraries.html) - defines a library 
- [`predicate`](https://fuellabs.github.io/sway/master/book/sway-program-types/predicates.html) - Boolean based ownership check
- [`script`](https://fuellabs.github.io/sway/master/book/sway-program-types/scripts.html) - a runnable bytecode on the chain, which executes once to preform a task


### [Attribute Keywords](#attribute-keywords)

Keywords associated with defining the funcitonallity of attributes

- [`allow`](https://fuellabs.github.io/sway/master/book/reference/attributes.html#allow) - overrides check for dead code
- [`doc`](https://fuellabs.github.io/sway/master/book/reference/attributes.html#doc) - specifies documentation
- [`inline`](https://fuellabs.github.io/sway/master/book/reference/attributes.html#inline) - suggests that a copy of the attributed function should be placed in the caller
- [`payable`](https://fuellabs.github.io/sway/master/book/reference/attributes.html#payable) - implies method is payable for complie time
- [`read`]() - pulls stored variables from `storage`
- [`storage`](https://fuellabs.github.io/sway/master/book/reference/attributes.html#storage) - declaration that contains a list of stored variables
- [`test`](https://fuellabs.github.io/sway/master/book/reference/attributes.html#test) - marks a function to be executed as a test
- [`write`]() - pushes variables to be stored with `storage`