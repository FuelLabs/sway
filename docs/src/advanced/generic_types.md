# Generic Types
_N.B.: Generic types in Sway are a work in progress, and are currently only partially implemented._

## Basics 

In Sway, generic types follow a very similar pattern to those in Rust. Let's look as some example syntax, 
starting with a generic function:

```sway
fn noop<T>(argument: T) -> T {
  argument
}
```

Here, the `noop()` function trivially returns exactly what was given to it. `T` is a _type parameter_, and it saying
that this function exists for all types T. More formally, this function could be typed as:

```
noop :: ∀T. T -> T
```

Generic types are a way to refer to types _in general_, meaning without specifying a single type. Clearly, our `noop` function
would work with any type in the language, so we don't need to specify `noop(argument: u8) -> u8`, `noop(argument: u16) -> u16`, etc.


## Code Generation
One question that arises when dealing with generic types is: how does the assembly handle this? There are a few approaches to handling
generic types at the lowest level. Sway uses a technique called [monomorphization](https://en.wikipedia.org/wiki/Monomorphization). This
means that the generic function is compiled to a non-generic version for every type it is called on. In this way, generic functions are
purely shorthand for the sake of ergonomics.

## Trait Constraints

Of course, our `noop()` function is not useful. Often, a programmer will want to declare functions over a types which satisfy certain traits.
For example, let's try to implement the successor function, `successor()`, for all numeric types.

```sway
fn successor<T>(argument: T)
  where T: Add
{
    argument + 1
}
```

Run `forc build`, and you will get:
```
.. |
 9 |   where T: Add 
10 |   {
11 |     argument + 1                                        
   |                ^ Mismatched types: expected type "T" but saw type "u64"
12 |   }
13 |

```

This is because we don't know for a fact that `1`, which in this case defaulted to `1u64`, actually can be added to `T`. What if `T` is `f64`? Or `b256`? What does it mean to add `1u64` in these cases?

We can solve this problem with another trait constraint. We can only find the successor of some value of type `T` if that type `T` defines some incrementor. Let's make a trait:

```sway
trait Incrementable {
  /// Returns the value to add when calculating the successor of a value.
  fn incrementor() -> Self;
}
```

Now, we can modify our `successor()` function:

```sway
fn successor<T>(argument: T)
  where T: Add,
        T: Incrementable
{
    argument + ~T::incrementor()
}
```
_(There's a little bit of new syntax here. When directly referring to a type to execute a method from it, a tilde (`~`) is used, This may change in the future.)_


## Generic Structs and Enums
Just like functions, structs and enums can be generic. Let's take a look at the standard library version of `Option<T>`:

```sway
enum Option<T> {
  Some: T,
  None: ()
}
```

Just like an unconstrained generic function, this type exists for all (∀) types `T`. `Result<T, E>` is another example:

```sway
enum Result<T, E> {
  Ok: T,
  Err: E
}
```

Both generic enums and generic structs can be trait constrained, as well. Consider this struct:
```sway
struct Foo<T>
  where T: Add 
{
    field_one: T
}
```
