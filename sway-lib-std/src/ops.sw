library;

use ::primitives::*;
use ::registers::flags;
use ::flags::{disable_panic_on_overflow, panic_on_overflow_enabled, set_flags};

const MAX_U32_U64: u64 = __transmute::<u32, u64>(u32::max());
const MAX_U16_U64: u64 = __transmute::<u16, u64>(u16::max());

/// Trait for the addition of two values.
pub trait Add {
    /// Add two values of the same type.
    ///
    /// # Arguments
    ///
    /// * `other`: [Self] - The value to add to self.
    ///
    /// # Returns
    ///
    /// * [Self] - The result of the two values added.
    ///
    /// # Examples
    ///
    /// ```sway
    /// struct MyStruct {
    ///     val: u64,
    /// }
    ///
    /// impl Add for MyStruct {
    ///     fn add(self, other: Self) -> Self {
    ///         let val = self.val + other.val;
    ///         Self {
    ///             val
    ///         }
    ///     }
    /// }
    ///
    /// fn foo() {
    ///     let struct1 = MyStruct { val: 1 };
    ///     let struct2 = MyStruct { val: 2 };
    ///     let result_struct = struct1 + struct2;
    ///     assert(result_struct.val == 3);
    /// }
    /// ```
    fn add(self, other: Self) -> Self;
}

impl Add for u256 {
    fn add(self, other: Self) -> Self {
        __add(self, other)
    }
}

impl Add for u64 {
    fn add(self, other: Self) -> Self {
        __add(self, other)
    }
}

// Emulate overflowing arithmetic for non-64-bit integer types
impl Add for u32 {
    fn add(self, other: Self) -> Self {
        let res_u64 = __add(
            __transmute::<Self, u64>(self),
            __transmute::<Self, u64>(other),
        );

        if __gt(res_u64, MAX_U32_U64) {
            if panic_on_overflow_enabled() {
                __revert(0)
            } else {
                // overflow enabled
                // res % (Self::max() + 1)
                __transmute::<u64, Self>(__mod(res_u64, __add(MAX_U32_U64, 1)))
            }
        } else {
            __transmute::<u64, Self>(res_u64)
        }
    }
}

impl Add for u16 {
    fn add(self, other: Self) -> Self {
        let res_u64 = __add(
            __transmute::<Self, u64>(self),
            __transmute::<Self, u64>(other),
        );

        if __gt(res_u64, MAX_U16_U64) {
            if panic_on_overflow_enabled() {
                __revert(0)
            } else {
                // overflow enabled
                // res % (Self::max() + 1)
                __transmute::<u64, Self>(__mod(res_u64, __add(MAX_U16_U64, 1)))
            }
        } else {
            __transmute::<u64, Self>(res_u64)
        }
    }
}

impl Add for u8 {
    fn add(self, other: Self) -> Self {
        let res_u64 = __add(u8_as_u64(self), u8_as_u64(other));

        let max_u8_u64 = u8_as_u64(Self::max());

        if __gt(res_u64, max_u8_u64) {
            if panic_on_overflow_enabled() {
                __revert(0)
            } else {
                // overflow enabled
                // res % (Self::max() + 1)
                u64_as_u8(__mod(res_u64, __add(max_u8_u64, 1)))
            }
        } else {
            u64_as_u8(res_u64)
        }
    }
}

/// Trait for the subtraction of two values.
pub trait Subtract {
    /// Subtract two values of the same type.
    ///
    /// # Arguments
    ///
    /// * `other`: [Self] - The value to subtract from self.
    ///
    /// # Returns
    ///
    /// * [Self] - The result of the two values subtracted.
    ///
    /// # Examples
    ///
    /// ```sway
    /// struct MyStruct {
    ///     val: u64,
    /// }
    ///
    /// impl Subtract for MyStruct {
    ///     fn subtract(self, other: Self) -> Self {
    ///         let val = self.val - other.val;
    ///         Self {
    ///             val
    ///         }
    ///     }
    /// }
    ///
    /// fn foo() {
    ///     let struct1 = MyStruct { val: 3 };
    ///     let struct2 = MyStruct { val: 1 };
    ///     let result_struct = struct1 - struct2;
    ///     assert(result_struct.val == 2);
    /// }
    /// ```
    fn subtract(self, other: Self) -> Self;
}

impl Subtract for u256 {
    fn subtract(self, other: Self) -> Self {
        __sub(self, other)
    }
}

impl Subtract for u64 {
    fn subtract(self, other: Self) -> Self {
        __sub(self, other)
    }
}

impl Subtract for u32 {
    fn subtract(self, other: Self) -> Self {
        let res_u64 = __sub(
            __transmute::<Self, u64>(self),
            __transmute::<Self, u64>(other),
        );

        if __gt(res_u64, MAX_U32_U64) {
            if panic_on_overflow_enabled() {
                __revert(0)
            } else {
                // overflow enabled
                // res % (Self::max() + 1)
                __transmute::<u64, Self>(__mod(res_u64, __add(MAX_U32_U64, 1)))
            }
        } else {
            __transmute::<u64, Self>(res_u64)
        }
    }
}

impl Subtract for u16 {
    fn subtract(self, other: Self) -> Self {
        let res_u64 = __sub(
            __transmute::<Self, u64>(self),
            __transmute::<Self, u64>(other),
        );

        if __gt(res_u64, MAX_U16_U64) {
            if panic_on_overflow_enabled() {
                __revert(0)
            } else {
                // overflow enabled
                // res % (Self::max() + 1)
                __transmute::<u64, Self>(__mod(res_u64, __add(MAX_U16_U64, 1)))
            }
        } else {
            __transmute::<u64, Self>(res_u64)
        }
    }
}

impl Subtract for u8 {
    fn subtract(self, other: Self) -> Self {
        let res_u64 = __sub(u8_as_u64(self), u8_as_u64(other));

        let max_u8_u64 = u8_as_u64(Self::max());

        if __gt(res_u64, max_u8_u64) {
            if panic_on_overflow_enabled() {
                __revert(0)
            } else {
                // overflow enabled
                // res % (Self::max() + 1)
                u64_as_u8(__mod(res_u64, __add(max_u8_u64, 1)))
            }
        } else {
            u64_as_u8(res_u64)
        }
    }
}

/// Trait for the multiplication of two values.
pub trait Multiply {
    /// Multiply two values of the same type.
    ///
    /// # Arguments
    ///
    /// * `other`: [Self] - The value to multiply with self.
    ///
    /// # Returns
    ///
    /// * [Self] - The result of the two values multiplied.
    ///
    /// # Examples
    ///
    /// ```sway
    /// struct MyStruct {
    ///     val: u64,
    /// }
    ///
    /// impl Multiply for MyStruct {
    ///     fn multiply(self, other: Self) -> Self {
    ///         let val = self.val * other.val;
    ///         Self {
    ///             val
    ///         }
    ///     }
    /// }
    ///
    /// fn foo() {
    ///     let struct1 = MyStruct { val: 3 };
    ///     let struct2 = MyStruct { val: 2 };
    ///     let result_struct = struct1 * struct2;
    ///     assert(result_struct.val == 6);
    /// }
    /// ```
    fn multiply(self, other: Self) -> Self;
}

impl Multiply for u256 {
    fn multiply(self, other: Self) -> Self {
        __mul(self, other)
    }
}

impl Multiply for u64 {
    fn multiply(self, other: Self) -> Self {
        __mul(self, other)
    }
}

// Emulate overflowing arithmetic for non-64-bit integer types
impl Multiply for u32 {
    fn multiply(self, other: Self) -> Self {
        let res_u64 = __mul(
            __transmute::<Self, u64>(self),
            __transmute::<Self, u64>(other),
        );

        if __gt(res_u64, MAX_U32_U64) {
            if panic_on_overflow_enabled() {
                __revert(0)
            } else {
                // overflow enabled
                // res % (Self::max() + 1)
                __transmute::<u64, Self>(__mod(res_u64, __add(MAX_U32_U64, 1)))
            }
        } else {
            __transmute::<u64, Self>(res_u64)
        }
    }
}

impl Multiply for u16 {
    fn multiply(self, other: Self) -> Self {
        let res_u64 = __mul(
            __transmute::<Self, u64>(self),
            __transmute::<Self, u64>(other),
        );

        if __gt(res_u64, MAX_U16_U64) {
            if panic_on_overflow_enabled() {
                __revert(0)
            } else {
                // overflow enabled
                // res % (Self::max() + 1)
                __transmute::<u64, Self>(__mod(res_u64, __add(MAX_U16_U64, 1)))
            }
        } else {
            __transmute::<u64, Self>(res_u64)
        }
    }
}

impl Multiply for u8 {
    fn multiply(self, other: Self) -> Self {
        let res_u64 = __mul(u8_as_u64(self), u8_as_u64(other));

        let max_u8_u64 = u8_as_u64(Self::max());

        if __gt(res_u64, max_u8_u64) {
            if panic_on_overflow_enabled() {
                __revert(0)
            } else {
                // overflow enabled
                // res % (Self::max() + 1)
                u64_as_u8(__mod(res_u64, __add(max_u8_u64, 1)))
            }
        } else {
            u64_as_u8(res_u64)
        }
    }
}

/// Trait for the division of two values.
pub trait Divide {
    /// Divide two values of the same type.
    ///
    /// # Arguments
    ///
    /// * `other`: [Self] - The value to divide with self.
    ///
    /// # Returns
    ///
    /// * [Self] - The result of the two values divided.
    ///
    /// # Examples
    ///
    /// ```sway
    /// struct MyStruct {
    ///     val: u64,
    /// }
    ///
    /// impl Divide for MyStruct {
    ///     fn divide(self, other: Self) -> Self {
    ///         let val = self.val / other.val;
    ///         Self {
    ///             val
    ///         }
    ///     }
    /// }
    ///
    /// fn foo() {
    ///     let struct1 = MyStruct { val: 10 };
    ///     let struct2 = MyStruct { val: 2 };
    ///     let result_struct = struct1 / struct2;
    ///     assert(result_struct.val == 5);
    /// }
    /// ```
    fn divide(self, other: Self) -> Self;
}

impl Divide for u256 {
    fn divide(self, other: Self) -> Self {
        __div(self, other)
    }
}

impl Divide for u64 {
    fn divide(self, other: Self) -> Self {
        __div(self, other)
    }
}

// division for unsigned integers cannot overflow,
// but if signed integers are ever introduced,
// overflow needs to be handled, since
// Self::max() / -1 overflows
impl Divide for u32 {
    fn divide(self, other: Self) -> Self {
        __div(self, other)
    }
}

impl Divide for u16 {
    fn divide(self, other: Self) -> Self {
        __div(self, other)
    }
}

impl Divide for u8 {
    fn divide(self, other: Self) -> Self {
        __div(self, other)
    }
}

/// Trait for the modulo of two values.
pub trait Mod {
    /// Modulo two values of the same type.
    ///
    /// # Arguments
    ///
    /// * `other`: [Self] - The value to mod with self.
    ///
    /// # Returns
    ///
    /// * [Self] - The modulo of the two values.
    ///
    /// # Examples
    ///
    /// ```sway
    /// struct MyStruct {
    ///     val: u64,
    /// }
    ///
    /// impl Mod for MyStruct {
    ///     fn modulo(self, other: Self) -> Self {
    ///         let val = self.val % other.val;
    ///         Self {
    ///             val
    ///         }
    ///     }
    /// }
    ///
    /// fn foo() {
    ///     let struct1 = MyStruct { val: 10 };
    ///     let struct2 = MyStruct { val: 2 };
    ///     let result_struct = struct1 % struct2;
    ///     assert(result_struct.val == 0);
    /// }
    /// ```
    fn modulo(self, other: Self) -> Self;
}

impl Mod for u256 {
    fn modulo(self, other: Self) -> Self {
        __mod(self, other)
    }
}

impl Mod for u64 {
    fn modulo(self, other: Self) -> Self {
        __mod(self, other)
    }
}

impl Mod for u32 {
    fn modulo(self, other: Self) -> Self {
        __mod(self, other)
    }
}

impl Mod for u16 {
    fn modulo(self, other: Self) -> Self {
        __mod(self, other)
    }
}

impl Mod for u8 {
    fn modulo(self, other: Self) -> Self {
        __mod(self, other)
    }
}

/// Trait to invert a type.
pub trait Not {
    /// Inverts the value of the type.
    ///
    /// # Returns
    ///
    /// * [Self] - The result of the inverse.
    ///
    /// # Examples
    ///
    /// ```sway
    /// struct MyStruct {
    ///     val: bool,
    /// }
    ///
    /// impl Not for MyStruct {
    ///     fn not(self) -> Self {
    ///         Self {
    ///             val: !self.val,
    ///         }
    ///     }
    /// }
    ///
    /// fn foo() {
    ///     let struct = MyStruct { val: true };
    ///     let result_struct = !struct;
    ///     assert(!result_struct.val);
    /// }
    /// ```
    fn not(self) -> Self;
}

impl Not for bool {
    fn not(self) -> Self {
        __eq(self, false)
    }
}

impl Not for u256 {
    fn not(self) -> Self {
        __not(self)
    }
}

impl Not for b256 {
    fn not(self) -> Self {
        __not(self)
    }
}

impl Not for u64 {
    fn not(self) -> Self {
        __not(self)
    }
}

impl Not for u32 {
    fn not(self) -> Self {
        let v = __not(self);
        __and(v, u32::max())
    }
}

impl Not for u16 {
    fn not(self) -> Self {
        let v = __not(self);
        __and(v, u16::max())
    }
}

impl Not for u8 {
    fn not(self) -> Self {
        let v = __not(self);
        __and(v, u8::max())
    }
}

/// Trait for comparing type instances using the equality operator.
///
/// Implementing this trait provides `==` and `!=` operators on a type.
///
/// This trait allows comparisons for types that do not have a full equivalence relation.
/// In other words, it is not required that each instance of the type must be
/// equal to itself. While most of the types used in blockchain development do have this
/// property, called reflexivity, we can encounter types that are not reflexive.
///
/// A typical example of a type supporting partial equivalence, but not equivalence,
/// is a floating point number, where `NaN` is different from any other number,
/// including itself: `NaN != NaN`.
pub trait PartialEq {
    /// Evaluates if two values of the same type are equal.
    ///
    /// # Arguments
    ///
    /// * `other`: [Self] - The value of the same type.
    ///
    /// # Returns
    ///
    /// * [bool] - `true` if the values are equal, otherwise `false`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// struct MyStruct {
    ///     val: u64,
    /// }
    ///
    /// impl PartialEq for MyStruct {
    ///     fn eq(self, other: Self) -> bool {
    ///         self.val == other.val
    ///     }
    /// }
    ///
    /// fn foo() {
    ///     let struct1 = MyStruct { val: 2 };
    ///     let struct2 = MyStruct { val: 2 };
    ///     let result = struct1 == struct2;
    ///     assert(result);
    /// }
    /// ```
    fn eq(self, other: Self) -> bool;
} {
    /// Evaluates if two values of the same type are not equal.
    ///
    /// # Additional Information
    ///
    /// This function is inherited when `eq()` is implemented.
    ///
    /// # Arguments
    ///
    /// * `other`: [Self] - The value of the same type.
    ///
    /// # Returns
    ///
    /// * [bool] - `true` if the two values are not equal, otherwise `false`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// struct MyStruct {
    ///     val: u64,
    /// }
    ///
    /// impl PartialEq for MyStruct {
    ///     fn eq(self, other: Self) -> bool {
    ///          self.val == other.val
    ///     }
    /// }
    ///
    /// fn foo() {
    ///     let struct1 = MyStruct { val: 10 };
    ///     let struct2 = MyStruct { val: 2 };
    ///     let result = struct1 != struct2;
    ///     assert(result);
    /// }
    /// ```
    fn neq(self, other: Self) -> bool {
        (self.eq(other)).not()
    }
}

/// Trait for comparing type instances corresponding to equivalence relations.
///
/// The difference between [Eq] and [PartialEq] is the additional requirement for reflexivity.
/// [PartialEq] guarantees symmetry and transitivity, but not reflexivity.
///
/// E.g., a type that implements [PartialEq] guarantees that for all `a`, `b`, and `c`:
/// - `a == b` implies `b == a` (symmetry)
/// - `a == b` and `b == c` implies `a == c` (transitivity)
///
/// [Eq], additionally implies:
/// - `a == a` for every `a` (reflexivity)
///
/// Reflexivity property cannot be checked by the compiler, and therefore `Eq`
/// does not have any methods, but only [PartialEq] as a supertrait.
///
/// **Implementing [Eq] for a type that does not have reflexivity property is a logic error**.
pub trait Eq: PartialEq {
}

impl PartialEq for bool {
    fn eq(self, other: Self) -> bool {
        __eq(self, other)
    }
}

impl Eq for bool {}

impl PartialEq for u256 {
    fn eq(self, other: Self) -> bool {
        __eq(self, other)
    }
}

impl Eq for u256 {}

impl PartialEq for b256 {
    fn eq(self, other: Self) -> bool {
        __eq(self, other)
    }
}

impl Eq for b256 {}

impl PartialEq for u64 {
    fn eq(self, other: Self) -> bool {
        __eq(self, other)
    }
}

impl Eq for u64 {}

impl PartialEq for u32 {
    fn eq(self, other: Self) -> bool {
        __eq(self, other)
    }
}

impl Eq for u32 {}

impl PartialEq for u16 {
    fn eq(self, other: Self) -> bool {
        __eq(self, other)
    }
}

impl Eq for u16 {}

impl PartialEq for u8 {
    fn eq(self, other: Self) -> bool {
        __eq(self, other)
    }
}

impl Eq for u8 {}

impl PartialEq for () {
    fn eq(self, other: Self) -> bool {
        true
    }
}

impl Eq for () {}

impl<T> PartialEq for (T, )
where
    T: PartialEq,
{
    fn eq(self, other: Self) -> bool {
        self.0 == other.0
    }
}

impl<T> Eq for (T, )
where
    T: Eq,
{}

impl<T1, T2> PartialEq for (T1, T2)
where
    T1: PartialEq,
    T2: PartialEq,
{
    fn eq(self, other: Self) -> bool {
        self.0 == other.0 && self.1 == other.1
    }
}

impl<T1, T2> Eq for (T1, T2)
where
    T1: Eq,
    T2: Eq,
{}

impl<T1, T2, T3> PartialEq for (T1, T2, T3)
where
    T1: PartialEq,
    T2: PartialEq,
    T3: PartialEq,
{
    fn eq(self, other: Self) -> bool {
        self.0 == other.0 && self.1 == other.1 && self.2 == other.2
    }
}

impl<T1, T2, T3> Eq for (T1, T2, T3)
where
    T1: Eq,
    T2: Eq,
    T3: Eq,
{}

#[cfg(experimental_const_generics = true)]
impl<T, const N: u64> PartialEq for [T; N]
where
    T: PartialEq,
{
    fn eq(self, other: Self) -> bool {
        let mut i = 0;
        while __lt(i, N) {
            let a: T = *__elem_at(&self, i);
            let b: T = *__elem_at(&other, i);

            if !a.eq(b) {
                return false;
            }

            i = __add(i, 1);
        };

        true
    }
}

#[cfg(experimental_const_generics = true)]
impl<T, const N: u64> Eq for [T; N]
where
    T: Eq,
{}

#[cfg(experimental_const_generics = true)]
impl<const N: u64> PartialEq for str[N] {
    fn eq(self, other: Self) -> bool {
        asm(result, left: self, right: other, len: N) {
            meq result left right len;
            result: bool
        }
    }
}

#[cfg(experimental_const_generics = true)]
impl<const N: u64> Eq for str[N] {}

/// Trait to evaluate if one value is greater or less than another of the same type.
pub trait Ord {
    /// Evaluates if one value of the same type is greater than another.
    ///
    /// # Arguments
    ///
    /// * `other`: [Self] - The value of the same type.
    ///
    /// # Returns
    ///
    /// * [bool] - `true` if `self` is greater than `other`, otherwise `false`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// struct MyStruct {
    ///     val: u64,
    /// }
    ///
    /// impl Ord for MyStruct {
    ///     fn gt(self, other: Self) -> bool {
    ///         self.val > other.val
    ///     }
    /// }
    ///
    /// fn foo() {
    ///     let struct1 = MyStruct { val: 10 };
    ///     let struct2 = MyStruct { val: 2 };
    ///     let result = struct1 > struct2;
    ///     assert(result);
    /// }
    /// ```
    fn gt(self, other: Self) -> bool;

    /// Evaluates if one value of the same type is less than another.
    ///
    /// # Arguments
    ///
    /// * `other`: [Self] - The value of the same type.
    ///
    /// # Returns
    ///
    /// * [bool] - `true` if `self` is less than `other`, otherwise `false`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// struct MyStruct {
    ///     val: u64,
    /// }
    ///
    /// impl Ord for MyStruct {
    ///     fn lt(self, other: Self) -> bool {
    ///         self.val < other.val
    ///     }
    /// }
    ///
    /// fn foo() {
    ///     let struct1 = MyStruct { val: 10 };
    ///     let struct2 = MyStruct { val: 2 };
    ///     let result = struct1 < struct2;
    ///     assert(!result);
    /// }
    /// ```
    fn lt(self, other: Self) -> bool;
}

impl Ord for u256 {
    fn gt(self, other: Self) -> bool {
        __gt(self, other)
    }

    fn lt(self, other: Self) -> bool {
        __lt(self, other)
    }
}

impl Ord for b256 {
    fn gt(self, other: Self) -> bool {
        __gt(self, other)
    }

    fn lt(self, other: Self) -> bool {
        __lt(self, other)
    }
}

impl Ord for u64 {
    fn gt(self, other: Self) -> bool {
        __gt(self, other)
    }
    fn lt(self, other: Self) -> bool {
        __lt(self, other)
    }
}

impl Ord for u32 {
    fn gt(self, other: Self) -> bool {
        __gt(self, other)
    }
    fn lt(self, other: Self) -> bool {
        __lt(self, other)
    }
}

impl Ord for u16 {
    fn gt(self, other: Self) -> bool {
        __gt(self, other)
    }
    fn lt(self, other: Self) -> bool {
        __lt(self, other)
    }
}

impl Ord for u8 {
    fn gt(self, other: Self) -> bool {
        __gt(self, other)
    }
    fn lt(self, other: Self) -> bool {
        __lt(self, other)
    }
}

/// Trait to bitwise AND two values of the same type.
pub trait BitwiseAnd {
    /// Bitwise AND two values of the same type.
    ///
    /// # Arguments
    ///
    /// * `other`: [Self] - The value of the same type.
    ///
    /// # Returns
    ///
    /// * [Self] - The result of the bitwise AND of the two values.
    ///
    /// # Examples
    ///
    /// ```sway
    /// struct MyStruct {
    ///     val: u64,
    /// }
    ///
    /// impl BitwiseAnd for MyStruct {
    ///     fn binary_and(self, other: Self) -> Self {
    ///         let val = self.val & other.val;
    ///         Self {
    ///             val
    ///         }
    ///     }
    /// }
    ///
    /// fn foo() {
    ///     let struct1 = MyStruct { val: 10 };
    ///     let struct2 = MyStruct { val: 11 };
    ///     let result_struct = struct1 & struct2;
    ///     assert(result_struct.val == 10);
    /// }
    /// ```
    fn binary_and(self, other: Self) -> Self;
}

impl BitwiseAnd for u256 {
    fn binary_and(self, other: Self) -> Self {
        __and(self, other)
    }
}

impl BitwiseAnd for b256 {
    fn binary_and(self, other: Self) -> Self {
        __and(self, other)
    }
}

impl BitwiseAnd for u64 {
    fn binary_and(self, other: Self) -> Self {
        __and(self, other)
    }
}

impl BitwiseAnd for u32 {
    fn binary_and(self, other: Self) -> Self {
        __and(self, other)
    }
}

impl BitwiseAnd for u16 {
    fn binary_and(self, other: Self) -> Self {
        __and(self, other)
    }
}

impl BitwiseAnd for u8 {
    fn binary_and(self, other: Self) -> Self {
        __and(self, other)
    }
}

/// Trait to bitwise OR two values of the same type.
pub trait BitwiseOr {
    /// Bitwise OR two values of the same type.
    ///
    /// # Arguments
    ///
    /// * `other`: [Self] - The value of the same type.
    ///
    /// # Returns
    ///
    /// * [Self] - The result of the bitwise OR of the two values.
    ///
    /// # Examples
    ///
    /// ```sway
    /// struct MyStruct {
    ///     val: u64,
    /// }
    ///
    /// impl BitwiseOr for MyStruct {
    ///     fn binary_or(self, other: Self) -> Self {
    ///         let val = self.val | other.val;
    ///         Self {
    ///             val
    ///         }
    ///     }
    /// }
    ///
    /// fn foo() {
    ///     let struct1 = MyStruct { val: 10 };
    ///     let struct2 = MyStruct { val: 11 };
    ///     let result_struct = struct1 | struct2;
    ///     assert(result_struct.val == 11);
    /// }
    /// ```
    fn binary_or(self, other: Self) -> Self;
}

impl BitwiseOr for u256 {
    fn binary_or(self, other: Self) -> Self {
        __or(self, other)
    }
}

impl BitwiseOr for b256 {
    fn binary_or(self, other: Self) -> Self {
        __or(self, other)
    }
}

impl BitwiseOr for u64 {
    fn binary_or(self, other: Self) -> Self {
        __or(self, other)
    }
}

impl BitwiseOr for u32 {
    fn binary_or(self, other: Self) -> Self {
        __or(self, other)
    }
}

impl BitwiseOr for u16 {
    fn binary_or(self, other: Self) -> Self {
        __or(self, other)
    }
}

impl BitwiseOr for u8 {
    fn binary_or(self, other: Self) -> Self {
        __or(self, other)
    }
}

/// Trait to bitwise XOR two values of the same type.
pub trait BitwiseXor {
    /// Bitwise XOR two values of the same type.
    ///
    /// # Arguments
    ///
    /// * `other`: [Self] - The value of the same type.
    ///
    /// # Returns
    ///
    /// * [Self] - The result of the bitwise XOR of the two values.
    ///
    /// # Examples
    ///
    /// ```sway
    /// struct MyStruct {
    ///     val: u64,
    /// }
    ///
    /// impl BitwiseXOr for MyStruct {
    ///     fn binary_xor(self, other: Self) -> Self {
    ///         let val = self.val ^ other.val;
    ///         Self {
    ///             val
    ///         }
    ///     }
    /// }
    ///
    /// fn foo() {
    ///     let struct1 = MyStruct { val: 10 };
    ///     let struct2 = MyStruct { val: 11 };
    ///     let result_struct = struct1 ^ struct2;
    ///     assert(result_struct.val == 1);
    /// }
    /// ```
    fn binary_xor(self, other: Self) -> Self;
}

impl BitwiseXor for u256 {
    fn binary_xor(self, other: Self) -> Self {
        __xor(self, other)
    }
}

impl BitwiseXor for b256 {
    fn binary_xor(self, other: Self) -> Self {
        __xor(self, other)
    }
}

impl BitwiseXor for u64 {
    fn binary_xor(self, other: Self) -> Self {
        __xor(self, other)
    }
}

impl BitwiseXor for u32 {
    fn binary_xor(self, other: Self) -> Self {
        __xor(self, other)
    }
}

impl BitwiseXor for u16 {
    fn binary_xor(self, other: Self) -> Self {
        __xor(self, other)
    }
}

impl BitwiseXor for u8 {
    fn binary_xor(self, other: Self) -> Self {
        __xor(self, other)
    }
}

pub trait OrdEq: Ord + PartialEq {
} {
    /// Evaluates if one value of the same type is greater or equal to than another.
    ///
    /// # Additional Information
    ///
    /// This trait requires that the `Ord` and `Eq` traits are implemented.
    ///
    /// # Arguments
    ///
    /// * `other`: [Self] - The value of the same type.
    ///
    /// # Returns
    ///
    /// * [bool] - `true` if `self` is greater than or equal to `other`, otherwise `false`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// struct MyStruct {
    ///     val: u64,
    /// }
    ///
    /// impl Eq for MyStruct {
    ///     fn eq(self, other: Self) -> bool {
    ///         self.val == other.val
    ///     }
    /// }
    ///
    /// impl Ord for MyStruct {
    ///     fn gt(self, other: Self) -> bool {
    ///         self.val > other.val
    ///     }
    /// }
    ///
    /// impl OrdEq for MyStruct {}
    ///
    /// fn foo() {
    ///     let struct1 = MyStruct { val: 10 };
    ///     let struct2 = MyStruct { val: 10 };
    ///     let result = struct1 >= struct2;
    ///     assert(result);
    /// }
    /// ```
    fn ge(self, other: Self) -> bool {
        self.gt(other) || self.eq(other)
    }
    /// Evaluates if one value of the same type is less or equal to than another.
    ///
    /// # Additional Information
    ///
    /// This trait requires that the `Ord` and `Eq` traits are implemented.
    ///
    /// # Arguments
    ///
    /// * `other`: [Self] - The value of the same type.
    ///
    /// # Returns
    ///
    /// * [bool] - `true` if `self` is less than or equal to `other`, otherwise `false`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// struct MyStruct {
    ///     val: u64,
    /// }
    ///
    /// impl Eq for MyStruct {
    ///     fn eq(self, other: Self) -> bool {
    ///         self.val == other.val
    ///     }
    /// }
    ///
    /// impl Ord for MyStruct {
    ///     fn lt(self, other: Self) -> bool {
    ///         self.val < other.val
    ///     }
    /// }
    ///
    /// impl OrdEq for MyStruct {}
    ///
    /// fn foo() {
    ///     let struct1 = MyStruct { val: 10 };
    ///     let struct2 = MyStruct { val: 10 };
    ///     let result = struct1 <= struct2;
    ///     assert(result);
    /// }
    /// ```
    fn le(self, other: Self) -> bool {
        self.lt(other) || self.eq(other)
    }
}

impl OrdEq for u256 {}
impl OrdEq for u64 {}
impl OrdEq for u32 {}
impl OrdEq for u16 {}
impl OrdEq for u8 {}
impl OrdEq for b256 {}

/// Trait to bit shift a value.
pub trait Shift {
    /// Bit shift left by an amount.
    ///
    /// # Arguments
    ///
    /// * `other`: [u64] - The amount to bit shift by.
    ///
    /// # Returns
    ///
    /// * [Self] - The result of the value bit shifted to the left.
    ///
    /// # Examples
    ///
    /// ```sway
    /// struct MyStruct {
    ///     val: u64,
    /// }
    ///
    /// impl Shift for MyStruct {
    ///     fn lsh(self, other: u64) -> Self {
    ///         let val = self.val << other;
    ///         Self {
    ///             val
    ///         }
    ///     }
    /// }
    ///
    /// fn foo() {
    ///     let struct1 = MyStruct { val: 10 };
    ///     let result_struct = struct1 << 3;
    ///     assert(result_struct.val == 80);
    /// }
    /// ```
    fn lsh(self, other: u64) -> Self;

    /// Bit shift right by an amount.
    ///
    /// # Arguments
    ///
    /// * `other`: [u64] - The amount to bit shift by.
    ///
    /// # Returns
    ///
    /// * [Self] - The result of the value bit shifted to the right.
    ///
    /// # Examples
    ///
    /// ```sway
    /// struct MyStruct {
    ///     val: u64,
    /// }
    ///
    /// impl Shift for MyStruct {
    ///     fn rsh(self, other: u64) -> Self {
    ///         let val = self.val >> other;
    ///         Self {
    ///             val
    ///         }
    ///     }
    /// }
    ///
    /// fn foo() {
    ///     let struct1 = MyStruct { val: 10 };
    ///     let result_struct = struct1 >> 1;
    ///     assert(result_struct.val == 5);
    /// }
    /// ```
    fn rsh(self, other: u64) -> Self;
}

impl Shift for u256 {
    fn lsh(self, other: u64) -> Self {
        __lsh(self, other)
    }
    fn rsh(self, other: u64) -> Self {
        __rsh(self, other)
    }
}

impl Shift for b256 {
    fn lsh(self, other: u64) -> Self {
        __lsh(self, other)
    }

    fn rsh(self, other: u64) -> Self {
        __rsh(self, other)
    }
}

impl Shift for u64 {
    fn lsh(self, other: u64) -> Self {
        __lsh(self, other)
    }
    fn rsh(self, other: u64) -> Self {
        __rsh(self, other)
    }
}

impl Shift for u32 {
    fn lsh(self, other: u64) -> Self {
        // any non-64-bit value is compiled to a u64 value under-the-hood
        // so we need to clear upper bits here
        __and(__lsh(self, other), Self::max())
    }
    fn rsh(self, other: u64) -> Self {
        __rsh(self, other)
    }
}

impl Shift for u16 {
    fn lsh(self, other: u64) -> Self {
        __and(__lsh(self, other), Self::max())
    }
    fn rsh(self, other: u64) -> Self {
        __rsh(self, other)
    }
}

/// Trait to compare values of the same type.
pub trait TotalOrd {
    /// Finds the minimum value of two values of the same type.
    ///
    /// # Arguments
    ///
    /// * `other`: [Self] - The value of the same type.
    ///
    /// # Returns
    ///
    /// * Self - the minimum of the two values, or the same value if they are equal.
    ///
    /// # Examples
    ///
    /// ```sway
    /// struct MyStruct {
    ///     val: u64,
    /// }
    ///
    /// impl TotalOrd for MyStruct {
    ///     fn min(self, other: Self) -> Self {
    ///         if self.val < other.val { self } else { other }
    ///     }
    /// }
    ///
    /// fn foo() {
    ///     let struct1 = MyStruct { val: 10 };
    ///     let struct2 = MyStruct { val: 20 };
    ///     let min = struct1.min(struct2);
    ///     assert(min.val == struct1.val);
    /// }
    /// ```
    fn min(self, other: Self) -> Self;
    /// Finds the maximum value of two values of the same type.
    ///
    /// # Arguments
    ///
    /// * `other`: [Self] - The value of the same type.
    ///
    /// # Returns
    ///
    /// * Self - the maximum of the two values, or the same value if they are equal.
    ///
    /// # Examples
    ///
    /// ```sway
    /// struct MyStruct {
    ///     val: u64,
    /// }
    ///
    /// impl TotalOrd for MyStruct {
    ///     fn max(self, other: Self) -> Self {
    ///         if self.val > other.val { self } else { other }
    ///     }
    /// }
    ///
    /// fn foo() {
    ///     let struct1 = MyStruct { val: 10 };
    ///     let struct2 = MyStruct { val: 20 };
    ///     let max = struct1.max(struct2);
    ///     assert(max.val == struct2.val);
    /// }
    /// ```
    fn max(self, other: Self) -> Self;
}

impl TotalOrd for u8 {
    fn min(self, other: Self) -> Self {
        if self < other { self } else { other }
    }

    fn max(self, other: Self) -> Self {
        if self > other { self } else { other }
    }
}

impl TotalOrd for u16 {
    fn min(self, other: Self) -> Self {
        if self < other { self } else { other }
    }

    fn max(self, other: Self) -> Self {
        if self > other { self } else { other }
    }
}

impl TotalOrd for u32 {
    fn min(self, other: Self) -> Self {
        if self < other { self } else { other }
    }

    fn max(self, other: Self) -> Self {
        if self > other { self } else { other }
    }
}

impl TotalOrd for u64 {
    fn min(self, other: Self) -> Self {
        if self < other { self } else { other }
    }

    fn max(self, other: Self) -> Self {
        if self > other { self } else { other }
    }
}

impl TotalOrd for u256 {
    fn min(self, other: Self) -> Self {
        if self < other { self } else { other }
    }

    fn max(self, other: Self) -> Self {
        if self > other { self } else { other }
    }
}

impl Shift for u8 {
    fn lsh(self, other: u64) -> Self {
        __and(__lsh(self, other), Self::max())
    }
    fn rsh(self, other: u64) -> Self {
        __rsh(self, other)
    }
}

/////////////////////////////////////////////////
// Internal Helpers
/////////////////////////////////////////////////

/// Build a single b256 value from a tuple of 4 u64 values.
fn compose(words: (u64, u64, u64, u64)) -> b256 {
    asm(r1: words) {
        r1: b256
    }
}

/// Get a tuple of 4 u64 values from a single b256 value.
fn decompose(val: b256) -> (u64, u64, u64, u64) {
    asm(r1: val) {
        r1: (u64, u64, u64, u64)
    }
}

#[test]
fn test_compose() {
    let expected: b256 = 0x0000000000000001_0000000000000002_0000000000000003_0000000000000004;
    let composed = compose((1, 2, 3, 4));
    if composed.neq(expected) {
        __revert(0)
    }
}

#[test]
fn test_decompose() {
    let initial: b256 = 0x0000000000000001_0000000000000002_0000000000000003_0000000000000004;
    let expected = (1, 2, 3, 4);
    let decomposed = decompose(initial);
    if decomposed.0.neq(expected.0)
        && decomposed.1.neq(expected.1)
        && decomposed.2.neq(expected.2)
        && decomposed.3.neq(expected.3)
    {
        __revert(0)
    }
}

use ::str::*;

impl PartialEq for str {
    fn eq(self, other: Self) -> bool {
        if self.len() != other.len() {
            false
        } else {
            let self_ptr = self.as_ptr();
            let other_ptr = other.as_ptr();
            let l = self.len();
            asm(r1: self_ptr, r2: other_ptr, r3: l, r4) {
                meq r4 r1 r2 r3;
                r4: bool
            }
        }
    }
}

impl Eq for str {}

impl u8 {
    /// Wrapping (modular) addition. Computes `self + other`, wrapping around at the boundary of the type.
    pub fn wrapping_add(self, other: Self) -> Self {
        let f = disable_panic_on_overflow();
        let res = self + other;
        set_flags(f);
        res
    }
    /// Wrapping (modular) subtraction. Computes `self - other`, wrapping around at the boundary of the type.
    pub fn wrapping_sub(self, other: Self) -> Self {
        let f = disable_panic_on_overflow();
        let res = self - other;
        set_flags(f);
        res
    }
    /// Wrapping (modular) multiplication. Computes `self * other`, wrapping around at the boundary of the type.
    pub fn wrapping_mul(self, other: Self) -> Self {
        let f = disable_panic_on_overflow();
        let res = self * other;
        set_flags(f);
        res
    }

    /// Returns whether a `u8` is set to zero.
    ///
    /// # Returns
    ///
    /// * [bool] -> True if the `u8` is zero, otherwise false.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let zero_u8 = u8::zero();
    ///     assert(zero_u8.is_zero());
    /// }
    /// ```
    pub fn is_zero(self) -> bool {
        self == 0u8
    }
}

impl u16 {
    /// Wrapping (modular) addition. Computes `self + other`, wrapping around at the boundary of the type.
    pub fn wrapping_add(self, other: Self) -> Self {
        let f = disable_panic_on_overflow();
        let res = self + other;
        set_flags(f);
        res
    }
    /// Wrapping (modular) subtraction. Computes `self - other`, wrapping around at the boundary of the type.
    pub fn wrapping_sub(self, other: Self) -> Self {
        let f = disable_panic_on_overflow();
        let res = self - other;
        set_flags(f);
        res
    }
    /// Wrapping (modular) multiplication. Computes `self * other`, wrapping around at the boundary of the type.
    pub fn wrapping_mul(self, other: Self) -> Self {
        let f = disable_panic_on_overflow();
        let res = self * other;
        set_flags(f);
        res
    }

    /// Returns whether a `u16` is set to zero.
    ///
    /// # Returns
    ///
    /// * [bool] -> True if the `u16` is zero, otherwise false.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let zero_u16 = u16::zero();
    ///     assert(zero_u16.is_zero());
    /// }
    /// ```
    pub fn is_zero(self) -> bool {
        self == 0u16
    }
}

impl u32 {
    /// Wrapping (modular) addition. Computes `self + other`, wrapping around at the boundary of the type.
    pub fn wrapping_add(self, other: Self) -> Self {
        let f = disable_panic_on_overflow();
        let res = self + other;
        set_flags(f);
        res
    }
    /// Wrapping (modular) subtraction. Computes `self - other`, wrapping around at the boundary of the type.
    pub fn wrapping_sub(self, other: Self) -> Self {
        let f = disable_panic_on_overflow();
        let res = self - other;
        set_flags(f);
        res
    }
    /// Wrapping (modular) multiplication. Computes `self * other`, wrapping around at the boundary of the type.
    pub fn wrapping_mul(self, other: Self) -> Self {
        let f = disable_panic_on_overflow();
        let res = self * other;
        set_flags(f);
        res
    }

    /// Returns whether a `u32` is set to zero.
    ///
    /// # Returns
    ///
    /// * [bool] -> True if the `u32` is zero, otherwise false.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let zero_u32 = u32::zero();
    ///     assert(zero_u32.is_zero());
    /// }
    /// ```
    pub fn is_zero(self) -> bool {
        self == 0u32
    }
}

impl u64 {
    /// Wrapping (modular) addition. Computes `self + other`, wrapping around at the boundary of the type.
    pub fn wrapping_add(self, other: Self) -> Self {
        let f = disable_panic_on_overflow();
        let res = self + other;
        set_flags(f);
        res
    }
    /// Wrapping (modular) subtraction. Computes `self - other`, wrapping around at the boundary of the type.
    pub fn wrapping_sub(self, other: Self) -> Self {
        let f = disable_panic_on_overflow();
        let res = self - other;
        set_flags(f);
        res
    }
    /// Wrapping (modular) multiplication. Computes `self * other`, wrapping around at the boundary of the type.
    pub fn wrapping_mul(self, other: Self) -> Self {
        let f = disable_panic_on_overflow();
        let res = self * other;
        set_flags(f);
        res
    }

    /// Returns whether a `u64` is set to zero.
    ///
    /// # Returns
    ///
    /// * [bool] -> True if the `u64` is zero, otherwise false.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let zero_u64 = u64::zero();
    ///     assert(zero_u64.is_zero());
    /// }
    /// ```
    pub fn is_zero(self) -> bool {
        self == 0u64
    }
}

impl u256 {
    /// Wrapping (modular) addition. Computes `self + other`, wrapping around at the boundary of the type.
    pub fn wrapping_add(self, other: Self) -> Self {
        let f = disable_panic_on_overflow();
        let res = self + other;
        set_flags(f);
        res
    }
    /// Wrapping (modular) subtraction. Computes `self - other`, wrapping around at the boundary of the type.
    pub fn wrapping_sub(self, other: Self) -> Self {
        let f = disable_panic_on_overflow();
        let res = self - other;
        set_flags(f);
        res
    }
    /// Wrapping (modular) multiplication. Computes `self * other`, wrapping around at the boundary of the type.
    pub fn wrapping_mul(self, other: Self) -> Self {
        let f = disable_panic_on_overflow();
        let res = self * other;
        set_flags(f);
        res
    }

    /// Returns whether a `u256` is set to zero.
    ///
    /// # Returns
    ///
    /// * [bool] -> True if the `u256` is zero, otherwise false.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let zero_u256 = u256::zero();
    ///     assert(zero_u256.is_zero());
    /// }
    /// ```
    pub fn is_zero(self) -> bool {
        self == 0x00u256
    }
}

impl b256 {
    /// Returns whether a `b256` is set to zero.
    ///
    /// # Returns
    ///
    /// * [bool] -> True if the `b256` is zero, otherwise false.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let zero_b256 = b256::zero();
    ///     assert(zero_b256.is_zero());
    /// }
    /// ```
    pub fn is_zero(self) -> bool {
        self == 0x0000000000000000000000000000000000000000000000000000000000000000
    }
}

fn u8_as_u64(val: u8) -> u64 {
    asm(input: val) {
        input: u64
    }
}

fn u64_as_u8(val: u64) -> u8 {
    asm(input: val) {
        input: u8
    }
}

#[cfg(experimental_const_generics = true)]
#[test]
fn ok_array_eq() {
    let a = [1, 2, 3];
    let b = [1, 2, 3];
    let c = [1, 1, 1];

    if !a.eq(a) {
        __revert(0);
    }

    if !a.eq(b) {
        __revert(0);
    }
    if !b.eq(a) {
        __revert(0);
    }

    if a.eq(c) {
        __revert(0);
    }
    if c.eq(a) {
        __revert(0);
    }
}
