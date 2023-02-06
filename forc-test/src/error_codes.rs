use thiserror::Error;

/// Revert with this value for a failing call to `std::revert::require`.
pub const FAILED_REQUIRE_SIGNAL: u64 = 18446744073709486080;

/// Revert with this value for a failing call to `std::token::transfer_to_address`.
pub const FAILED_TRANSFER_TO_ADDRESS_SIGNAL: u64 = 18446744073709486081;

/// Revert with this value for a failing call to `std::message::send_message`.
pub const FAILED_SEND_MESSAGE_SIGNAL: u64 = 18446744073709486082;

/// Revert with this value for a failing call to `std::assert::assert_eq`.
pub const FAILED_ASSERT_EQ_SIGNAL: u64 = 18446744073709486083;

/// Revert with this value for a failing call to `std::assert::assert`.
pub const FAILED_ASSERT_SIGNAL: u64 = 18446744073709486084;


#[derive(Error, Debug)]
pub enum ErrorSignal {
    #[error("Failing call to `std::revert::require`")]
    Require,
    #[error("Failing call to `std::token::transfer_to_address`")]
    TransferToAddress,
    #[error("Failing call to `std::message::send_message`")]
    SendMessage,
    #[error("Failing call to `std::assert::assert_eq`")]
    AssertEq,
    #[error("Failing call to `std::assert::assert`")]
    Assert,
    #[error("Unknown error signal")]
    Unknown,
}



