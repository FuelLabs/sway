//! Values which signify special types of errors when passed to `std::revert::revert`.
library;

/// A revert with this value signals that it was caused by a failing call to `std::revert::require`.
///
/// # Additional Information
///
/// The value is: 18446744073709486080
pub const FAILED_REQUIRE_SIGNAL = 0xffff_ffff_ffff_0000;

/// A revert with this value signals that it was caused by a failing call to `std::asset::transfer_to_address`.
///
/// # Additional Information
///
/// The value is: 18446744073709486081
pub const FAILED_TRANSFER_TO_ADDRESS_SIGNAL = 0xffff_ffff_ffff_0001;

/// A revert with this value signals that it was caused by a failing call to `std::assert::assert_eq`.
///
/// # Additional Information
///
/// The value is: 18446744073709486083
pub const FAILED_ASSERT_EQ_SIGNAL = 0xffff_ffff_ffff_0003;

/// A revert with this value signals that it was caused by a failing call to `std::assert::assert`.
///
/// # Additional Information
///
/// The value is: 18446744073709486084
pub const FAILED_ASSERT_SIGNAL = 0xffff_ffff_ffff_0004;

/// A revert with this value signals that it was caused by a failing call to `std::assert::assert_ne`.
///
/// # Additional Information
///
/// The value is: 18446744073709486085
pub const FAILED_ASSERT_NE_SIGNAL = 0xffff_ffff_ffff_0005;

/// A revert with this value signals that it was caused by a call to `std::revert::revert_with_log`.
///
/// # Additional Information
///
/// The value is: 18446744073709486086
pub const REVERT_WITH_LOG_SIGNAL = 0xffff_ffff_ffff_0006;

/// A revert with this value signals that it was caused by a call to `raw_slice::len::<T>` with T having length zero.
///
/// # Additional Information
///
/// The value is: 18446744073709486087
pub const REVERT_WITH_RAW_SLICE_LEN_ZST = 0xffff_ffff_ffff_0007;

/// A revert with this value signals that it was caused by a call to `TrivialBool::unwrap`.
///
/// # Additional Information
///
/// The value is: 18446744073709486088
pub const REVERT_WITH_TRIVIAL_BOOL_UNWRAP = 0xffff_ffff_ffff_0008;

/// A revert with this value signals that it was caused by a call to `TrivialEnum::unwrap`.
///
/// # Additional Information
///
/// The value is: 18446744073709486089
pub const REVERT_WITH_TRIVIAL_ENUM_UNWRAP = 0xffff_ffff_ffff_0009;
