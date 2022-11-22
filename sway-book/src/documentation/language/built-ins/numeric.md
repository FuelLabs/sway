# Numeric Types

Broadly speaking there are two types of integers:

<!-- no toc -->
- [Signed](#signed-integers) (positive and negative)
- [Unsigned](#unsigned-integers) (only positive)

## Signed Integers

A signed integer is a whole number which can take the value of zero and both negative and positive values. This means that a signed integer can take values such as:

- -42
- 0
- 42

In order to achieve this one _bit_ must be kept for tracking the sign (+ or -) of the value and thus the range of available values is smaller than an unsigned integer.

For those inclined, the range for an n-bit signed integer is -2<sup>n-1</sup> to 2<sup>n-1</sup>-1.

Sway does not natively support signed integers however there is nothing stopping a library from using primitives to create types that act like signed types.

## Unsigned Integers

An unsigned integer is a whole number which can take the value of zero or any positive number, but cannot be negative. This allows for one more _bit_ of values to be used for the positive numbers and thus the positive range is significantly larger than for signed integers.

An example of available values is:

- 0
- 42

For those inclined, the range for an n-bit unsigned integer is 0 to 2<sup>n</sup>-1.

## Alternative Syntax

All of the unsigned integer types are numeric types, and the `byte` type can also be viewed as an 8-bit unsigned integer.

Numbers can be declared with binary syntax, hexadecimal syntax, base-10 syntax, and underscores for delineation.

```sway
{{#include ../../../code/language/built-ins/numerics/src/lib.sw:syntax}}
```
