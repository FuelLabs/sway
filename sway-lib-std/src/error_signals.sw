//! Values which signify special types of errors when passed to revert()
library error_signals;

/// revert with this value for a failing call to std::revert::require.
pub const FAILED_REQUIRE_SIGNAL = 42;
/// revert with this value for a failing call to std::message::send_message.
pub const FAILED_SEND_MESSAGE_SIGNAL = 0xffff_ffff_ffff_0002;
