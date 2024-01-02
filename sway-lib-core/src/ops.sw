library;

use ::primitives::*;

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
        // any non-64-bit value is compiled to a u64 value under-the-hood
        // constants (like Self::max() below) are also automatically promoted to u64
        let res = __add(self, other);
        if __gt(res, Self::max()) {
            // integer overflow
            __revert(0)
        } else {
            // no overflow
            res
        }
    }
}

impl Add for u16 {
    fn add(self, other: Self) -> Self {
        let res = __add(self, other);
        if __gt(res, Self::max()) {
            __revert(0)
        } else {
            res
        }
    }
}

impl Add for u8 {
    fn add(self, other: Self) -> Self {
        let self_u64 = asm(input: self) {
            input: u64
        };
        let other_u64 = asm(input: other) {
            input: u64
        };
        let res_u64 = __add(self_u64, other_u64);
        let max_u8_u64 = asm(input: Self::max()) {
            input: u64
        };
        if __gt(res_u64, max_u8_u64) {
            __revert(0)
        } else {
            asm(input: res_u64) {
                input: u8
            }
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

// unlike addition, underflowing subtraction does not need special treatment
// because VM handles underflow
impl Subtract for u32 {
    fn subtract(self, other: Self) -> Self {
        __sub(self, other)
    }
}

impl Subtract for u16 {
    fn subtract(self, other: Self) -> Self {
        __sub(self, other)
    }
}

impl Subtract for u8 {
    fn subtract(self, other: Self) -> Self {
        __sub(self, other)
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
        // any non-64-bit value is compiled to a u64 value under-the-hood
        // constants (like Self::max() below) are also automatically promoted to u64
        let res = __mul(self, other);
        if __gt(res, Self::max()) {
            // integer overflow
            __revert(0)
        } else {
            // no overflow
            res
        }
    }
}

impl Multiply for u16 {
    fn multiply(self, other: Self) -> Self {
        let res = __mul(self, other);
        if __gt(res, Self::max()) {
            __revert(0)
        } else {
            res
        }
    }
}

impl Multiply for u8 {
    fn multiply(self, other: Self) -> Self {
        let self_u64 = asm(input: self) {
            input: u64
        };
        let other_u64 = asm(input: other) {
            input: u64
        };
        let res_u64 = __mul(self_u64, other_u64);
        let max_u8_u64 = asm(input: Self::max()) {
            input: u64
        };
        if __gt(res_u64, max_u8_u64) {
            __revert(0)
        } else {
            asm(input: res_u64) {
                input: u8
            }
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

/// Trait to evaluate if two types are equal.
pub trait Eq {
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
    /// impl Eq for MyStruct {
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
    /// impl Eq for MyStruct {
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

impl Eq for bool {
    fn eq(self, other: Self) -> bool {
        __eq(self, other)
    }
}

impl Eq for u256 {
    fn eq(self, other: Self) -> bool {
        __eq(self, other)
    }
}

impl Eq for b256 {
    fn eq(self, other: Self) -> bool {
        __eq(self, other)
    }
}

impl Eq for u64 {
    fn eq(self, other: Self) -> bool {
        __eq(self, other)
    }
}

impl Eq for u32 {
    fn eq(self, other: Self) -> bool {
        __eq(self, other)
    }
}

impl Eq for u16 {
    fn eq(self, other: Self) -> bool {
        __eq(self, other)
    }
}

impl Eq for u8 {
    fn eq(self, other: Self) -> bool {
        __eq(self, other)
    }
}

impl Eq for raw_ptr {
    fn eq(self, other: Self) -> bool {
        __eq(self, other)
    }
}

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

/// Trait to evaluate if one value is greater than or equal, or less than or equal to another of the same type.
trait OrdEq: Ord + Eq {
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

impl Eq for str {
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

fn assert(v: bool) {
    if !v {
        __revert(0)
    }
}

#[test]
pub fn ok_str_eq() {
    assert("" == "");
    assert("a" == "a");
    
    assert("a" != "");
    assert("" != "a");
    assert("a" != "b");
}