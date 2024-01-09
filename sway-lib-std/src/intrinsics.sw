//! Exposes compiler intrinsics as stdlib wrapper functions.
library;

/// Returns whether a generic type `T` is a reference type or not.
///
/// # Returns
///
/// * [bool] - `true` if `T` is a reference type, `false` otherwise.
///
/// # Examples
///
/// ```sway
/// use std::intrinsics::is_reference_type;
///
/// fn foo() {
///     let a = 1;
///     assert(is_reference_type(a))
/// }
/// ```
pub fn is_reference_type<T>() -> bool {
    __is_reference_type::<T>()
}

/// Returns the size of a generic type `T` in bytes.
///
/// # Returns
///
/// * [u64] - The size of `T` in bytes.
///
/// # Examples
///
/// ```sway
/// use std::intrinsics::size_of;
///
/// fn foo() {
///     assert(size_of::<u64>() == 8);
/// }
/// ```
///
/// ```sway
/// use std::intrinsics::size_of;
///
/// pub struct Foo {
///     a: u64,
///     b: u64,
/// }
///
/// fn foo() {
///     assert(size_of::<Foo>() == 16);
/// }
/// ```
pub fn size_of<T>() -> u64 {
    __size_of::<T>()
}

/// Returns the size of the type of a value in bytes.
///
/// # Arguments
///
/// * `val` - The value to get the size of.
///
/// # Returns
///
/// * [u64] - The size of the type of `val` in bytes.
///
/// # Examples
///
/// ```sway
/// use std::intrinsics::size_of_val;
///
/// fn foo() {
///     let a = 1;
///     assert(size_of_val(a) == 8);
/// }
/// ```
///
/// ```sway
/// use std::intrinsics::size_of_val;
///
/// pub struct Foo {
///     a: u64,
///     b: u64,
/// }
///
/// fn foo() {
///     let a = Foo { a: 1, b: 2 };
///     assert(size_of_val(a) == 16);
/// }
/// ```
pub fn size_of_val<T>(val: T) -> u64 {
    __size_of_val::<T>(val)
}
