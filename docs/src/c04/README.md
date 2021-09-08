# Sway on the Chain

Sway is fundamentally a blockchain language. Because of this, it has some features and requirements that you may not have seen in general purpose programming languages.

## Program Types

A Sway program itself has a type: it is either a _contract_, _predicate_, _script_, or _library_. The first three of these things are all deployable to the blockchain. A _library_ is simply a project designed for code reuse, and is never directly deployed to the chain.

Every Sway file _must_ begin with a declaration of what type of program it is. A program can have many libraries within it, but only one contract, script, or predicate. Scripts and predicates require `main` functions to serve as entry points, while contracts instead publish an ABI. This chapter will go into detail about all of these various types of programs and what purposes they serve. For now, let's take a look at these types of programs, to get a feel of the syntax.

### Script

```sway
script;

// All scripts require a main function.
fn main() {
  return;
}
```

### Predicate

```sway
predicate;

// All predicates require a main function which returns a boolean value.
fn main() -> bool {
  return true;
}
```

### Contract

```sway
contract;

// Public functions in contracts are compiled into an ABI, which can be called
// from other code deployed on the chain.
pub fn entry_one(arg: u64) -> u64 {
  arg
}
```

### Library

```sway
// note that libraries must be named, so we know how to refer to them and import things.
library my_library;

// All public items in a library are made available to other projects which import this library.
pub struct MyStruct {
  field_one: u64,
  field_two: bool,
}
```
