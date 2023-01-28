//! Values which signify special types of errors when passed to `std::revert::revert`.
library error_signals;

/// Revert with this value for a failing call to `std::revert::require`.
/// 18446744073709486080
pub const FAILED_REQUIRE_SIGNAL = 0xffff_ffff_ffff_0000;

/// Revert with this value for a failing call to `std::token::transfer_to_address`.
/// 18446744073709486081
pub const FAILED_TRANSFER_TO_ADDRESS_SIGNAL = 0xffff_ffff_ffff_0001;

/// Revert with this value for a failing call to `std::message::send_message`.
/// 18446744073709486082
pub const FAILED_SEND_MESSAGE_SIGNAL = 0xffff_ffff_ffff_0002;

/// Revert with this value for a failing call to `std::assert::assert_eq`.
/// 18446744073709486083
pub const FAILED_ASSERT_EQ_SIGNAL = 0xffff_ffff_ffff_0003;
