# Variables

A variable is a way to reference some information by a specific name and it can take the form of a variety of [data structures](../built-ins/index.md).

In Sway there are two states that a variable can take:

- Immutable
  - Can be read but cannot be changed after it has been declared
- Mutable
  - Can be read and can have its value changed if and only if the new value is the same type

By default all variables in Sway are immutable unless declared as mutable through the use of the `mut` keyword. This is one of the ways in which Sway encourages safe programming, and many modern languages have the same default.

In the following sections, we'll take a look at two keywords that are used to instantiate information ([let](let.md) & [const](const.md)) and a way to temporarily reuse a variable name without affecting the original instantiation through [variable shadowing](variable-shadowing.md).
