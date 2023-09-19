# Traits

A trait describes an abstract interface that types can implement. This interface consists of an `interface
surface` of associated items, along with `methods`.

```sway
trait Trait {
    fn fn_sig(self, b: Self) -> bool;
} {
    fn method(self, b: Self) -> bool {
        true
    }
}
```

Associated items come in two varieties:

- [Functions](#associated-functions)
- [Constants](#associated-constants)
- [Types](#associated-types)

All traits define an implicit type parameter `Self` that refers to "the type that is implementing this interface".
Traits may also contain additional type parameters. These type parameters, including `Self`, may be constrained by
other traits and so forth as usual.

Traits are implemented for specific types through separate implementations.

## Associated functions

Trait functions consist of just a function signature. This indicates that the implementation must define the function.

## Associated constants

Associated constants are constants associated with a type.

An *associated constant declaration* declares a signature for *associated constant definitions*.
It is written as `const`, then an identifier, then `:`, then a type, finished by a `;`.

The identifier is the name of the constant used in the path. The type is the type that the definition has to implement.

An *associated constant definition* defines a constant associated with a type.

### Associated constants examples

```sway
{{#include ../../../code/language/traits/associated-consts/src/lib.sw}}
```

Associated constants may omit the equals sign and expression to indicate implementations must define the constant value.

## Associated types

Associated types in Sway allow you to define placeholder types within a trait, which can be customized by concrete
implementations of that trait. These associated types are used to specify the return types of trait methods or to
define type relationships within the trait.

### Associated types examples

```sway
{{#include ../../../code/language/traits/associated-types/src/lib.sw}}
```
