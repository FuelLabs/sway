library;

use ::convert::{From, Into};
use ::ops::*;
use ::result::Result::{self, *};
use ::codec::*;
use ::hash::{Hash, Hasher};

const TAI_64_CONVERTER: u64 = 10 + (1 << 62);

/// A duration of time.
pub struct Duration {
    /// The underlying seconds of the duration.
    seconds: u64,
}

impl Duration {
    /// A duration of 0 seconds.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::time::Duration;
    ///
    /// fn foo() {
    ///     let zero_seconds = Duration::ZERO;
    ///     assert(zero_seconds.as_seconds() == 0u64);
    /// }
    /// ```
    pub const ZERO: Self = Self { seconds: 0 };

    /// The maximum duration.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::time::Duration;
    ///
    /// fn foo() {
    ///     let max_duration = Duration::MAX;
    ///     assert(max_duration.as_seconds() == u64::MAX);
    /// }
    /// ```
    pub const MAX: Self = Self {
        seconds: u64::max(),
    };

    /// The minimum duration.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::time::Duration;
    ///
    /// fn foo() {
    ///     let min_duration = Duration::MIN;
    ///     assert(min_duration.as_seconds() == u64::MIN);
    /// }
    /// ```
    pub const MIN: Self = Self {
        seconds: u64::min(),
    };

    /// One second of duration.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::time::Duration;
    ///
    /// fn foo() {
    ///     let 1_second = Duration::SECOND;
    ///     assert(1_second.as_seconds() == 1);
    /// }
    /// ```
    pub const SECOND: Self = Self { seconds: 1 };

    /// One minute of duration.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::time::MINUTE;
    ///
    ///  fn foo() {
    ///     let 1_minute = Duration::MINUTE;
    ///     assert(1_minute.as_minutes() == 1);
    /// }
    /// ```
    pub const MINUTE: Self = Self { seconds: 60 };

    /// One hour of duration.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::time::Duration;
    ///
    /// fn foo() {
    ///     let 1_hour = Duration::HOUR;
    ///     assert(1_hour.as_hours() == 1);
    /// }
    /// ```
    pub const HOUR: Self = Self { seconds: 3_600 };

    /// One day of duration.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::time::Duration;
    ///
    /// fn foo() {
    ///     let 1_day = Duration::DAY;
    ///     assert(1_day.as_days() == 1);
    /// }
    /// ```
    pub const DAY: Self = Self {
        seconds: 86_400,
    };

    /// 1 week of duration.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::time::Duration;
    ///
    /// fn foo() {
    ///     let 1_week = Duration::WEEK;
    ///     assert(1_week.as_weeks() == 1);
    /// }
    /// ```
    pub const WEEK: Self = Self {
        seconds: 604_800,
    };

    /// Creates a new `Duration` from a number of seconds.
    ///
    /// # Arguments
    ///
    /// * `seconds`: [u64] - The number of seconds from which to create a duration.
    ///
    /// # Returns
    ///
    /// * [Duration] - A new `Duration` with the specified number of seconds.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::time::Duration;
    ///
    /// fn foo() {
    ///     let 30_seconds = Duration::seconds(30);
    ///     assert(30_seconds.as_seconds() == 30);
    /// }
    /// ```
    pub fn seconds(seconds: u64) -> Self {
        Self { seconds }
    }

    /// Creates a new `Duration` from a number of minutes.
    ///
    /// # Arguments
    ///
    /// * `minutes`: [u64] - The number of minutes from which to create a duration.
    ///
    /// # Returns
    ///
    /// * [Duration] - A new `Duration` with the specified number of minutes.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::time::Duration;
    ///
    /// fn foo() {
    ///     let 30_minutes = Duration::minutes(30);
    ///     assert(30_minutes.as_minutes() == 30);
    /// }
    /// ```
    pub fn minutes(minutes: u64) -> Self {
        Self {
            seconds: minutes * 60,
        }
    }

    /// Creates a new `Duration` from a number of hours.
    ///
    /// # Arguments
    ///
    /// * `hours`: [u64] - The number of hours from which to create a duration.
    ///
    /// # Returns
    ///
    /// * [Duration] - A new `Duration` with the specified number of hours.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::time::Duration;
    ///
    /// fn foo() {
    ///     let 30_hours = Duration::hours(30);
    ///     assert(30_hours.as_hours() == 30);
    /// }
    /// ```
    pub fn hours(hours: u64) -> Self {
        Self {
            seconds: hours * 3_600,
        }
    }

    /// Creates a new `Duration` from a number of days.
    ///
    /// # Arguments
    ///
    /// * `days`: [u64] - The number of days from which to create a duration.
    ///
    /// # Returns
    ///
    /// * [Duration] - A new `Duration` with the specified number of days.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::time::Duration;
    ///
    /// fn foo() {
    ///     let 30_days = Duration::days(30);
    ///     assert(30_days.as_days() == 30);
    /// }
    /// ```
    pub fn days(days: u64) -> Self {
        Self {
            seconds: days * 86_400,
        }
    }

    /// Creates a new `Duration` from a number of weeks.
    ///
    /// # Arguments
    ///
    /// * `weeks`: [u64] - The number of weeks from which to create a duration.
    ///
    /// # Returns
    ///
    /// * [Duration] - A new `Duration` with the specified number of weeks.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::time::Duration;
    ///
    /// fn foo() {
    ///     let 30_weeks = Duration::weeks(30);
    ///     assert(30_weeks.as_weeks() == 30);
    /// }
    /// ```
    pub fn weeks(weeks: u64) -> Self {
        Self {
            seconds: weeks * 604_800,
        }
    }

    /// Returns the number of seconds in a `Duration`.
    ///
    /// # Returns
    ///
    /// * [u64] - The number of seconds in a `Duration`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::time::Duration;
    ///
    /// use fn foo() {
    ///     let 2_minutes = Duration::minutes(2);
    ///     let result_seconds = 2_minutes.as_seconds();
    ///     assert(result_seconds == 120);
    /// }
    /// ```
    pub fn as_seconds(self) -> u64 {
        self.seconds
    }

    /// Returns the number of minutes in a `Duration`.
    ///
    /// # Additional Information
    ///
    /// **Warning** If the duration is not perfectly divisible by a minute, a rounded value is returned.
    ///
    /// # Returns
    ///
    /// * [u64] - The number of minutes in a `Duration`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::time::Duration;
    ///
    /// use fn foo() {
    ///     let 2_hours = Duration::hours(2);
    ///     let result_minutes = 2_hours.as_minutes();
    ///     assert(result_minutes == 120);
    /// }
    /// ```
    pub fn as_minutes(self) -> u64 {
        self.seconds / 60
    }

    /// Returns the number of hours in a `Duration`.
    ///
    /// # Additional Information
    ///
    /// **Warning** If the duration is not perfectly divisible by an hour, a rounded value is returned.
    ///
    /// # Returns
    ///
    /// * [u64] - The number of hours in a `Duration`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::time::Duration;
    ///
    /// use fn foo() {
    ///     let 2_days = Duration::days(2);
    ///     let result_hours = 2_days.as_hours();
    ///     assert(result_hours == 48);
    /// }
    /// ```
    pub fn as_hours(self) -> u64 {
        self.seconds / 3_600
    }

    /// Returns the number of days in a `Duration`.
    ///
    /// # Additional Information
    ///
    /// **Warning** If the duration is not perfectly divisible by a day, a rounded value is returned.
    ///
    /// # Returns
    ///
    /// * [u64] - The number of days in a `Duration`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::time::Duration;
    ///
    /// use fn foo() {
    ///     let 2_weeks = Duration::weeks(2);
    ///     let result_days = 2_weeks.as_days();
    ///     assert(result_days == 14);
    /// }
    /// ```
    pub fn as_days(self) -> u64 {
        self.seconds / 86_400
    }

    /// Returns the number of weeks in a `Duration`.
    ///
    /// # Additional Information
    ///
    /// **Warning** If the duration is not perfectly divisible by a week, a rounded value is returned.
    ///
    /// # Returns
    ///
    /// * [u64] - The number of weeks in a `Duration`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::time::Duration;
    ///
    /// use fn foo() {
    ///     let 2_weeks = Duration::weeks(2);
    ///     let result_weeks = 2_weeks.as_weeks();
    ///     assert(result_weeks == 2);
    /// }
    /// ```
    pub fn as_weeks(self) -> u64 {
        self.seconds / 604_800
    }

    /// Returns whether the `Duration` is zero seconds.
    ///
    /// # Returns
    ///
    /// * [bool] - `true` if the `Duration` is zero, otherwise `false`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::time::Duration;
    ///
    /// fn foo() {
    ///     let zero_duration = Duration::ZERO;
    ///     assert(zero_duration.is_zero() == true);
    /// }
    /// ```
    pub fn is_zero(self) -> bool {
        self.seconds == 0
    }
}

impl Add<Self> for Duration {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self {
            seconds: self.seconds + other.seconds,
        }
    }
}

impl Subtract<Self> for Duration {
    type Output = Self;
    fn subtract(self, other: Self) -> Self {
        Self {
            seconds: self.seconds - other.seconds,
        }
    }
}

// impl Multiply for Duration {
//     fn multiply(self, other: Self) -> Self {
//         Self {
//             seconds: self.seconds * other.seconds,
//         }
//     }
// }

// impl Divide for Duration {
//     fn divide(self, other: Self) -> Self {
//         Self {
//             seconds: self.seconds / other.seconds,
//         }
//     }
// }


impl PartialEq for Duration {
    fn eq(self, other: Self) -> bool {
        self.seconds == other.seconds
    }
}

impl Eq for Duration {}

impl Ord for Duration {
    fn gt(self, other: Self) -> bool {
        self.seconds > other.seconds
    }

    fn lt(self, other: Self) -> bool {
        self.seconds < other.seconds
    }
}

impl OrdEq for Duration {}

impl Hash for Duration {
    fn hash(self, ref mut state: Hasher) {
        self.seconds.hash(state);
    }
}

impl From<u64> for Duration {
    fn from(seconds: u64) -> Self {
        Self { seconds }
    }
}

impl Into<u64> for Duration {
    fn into(self) -> u64 {
        self.seconds
    }
}

/// Returned when something fails when computing time.
pub enum TimeError {
    /// Returned when the `Time` passed is later than the current `Time`.
    LaterThanTime: (),
    /// Returned when the current `Time` is later than the current block time.
    LaterThanNow: (),
}

/// A UNIX timestamp.
pub struct Time {
    /// The underlying UNIX timestamp.
    unix: u64,
}

impl Time {
    /// Creates a new UNIX `Time`.
    ///
    /// # Arguments
    ///
    /// * `unix_timestamp`: [u64] - A UNIX timestamp represented as a `u64`.
    ///
    /// # Returns
    ///
    /// * [Time] - A new UNIX `Time`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::time::Time;
    ///
    /// fn foo(unix_timestamp: u64) {
    ///     let new_time = Time::new(unix_timestamp);
    ///     assert(new_time.is_zero() == false);
    /// }
    /// ```
    pub fn new(unix_timestamp: u64) -> Self {
        Self {
            unix: unix_timestamp,
        }
    }

    /// Returns the UNIX time of the current block.
    ///
    /// # Returns
    ///
    /// * [Time] - The current UNIX time.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::time::Time;
    ///
    /// fn foo() {
    ///     let now = Time::now();
    ///     assert(now.is_zero() == false);
    /// }
    /// ```
    pub fn now() -> Self {
        let tia64 = asm(timestamp, height) {
            bhei height;
            time timestamp height;
            timestamp: u64
        };

        Self {
            unix: tia64 - TAI_64_CONVERTER,
        }
    }

    /// Returns the UNIX time of a specific block.
    ///
    /// # Arguments
    ///
    /// * `block_height`: [u32] - The block which the time should be returned.
    ///
    /// # Returns
    ///
    /// * [Time] - The UNIX time of the specified block.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::{time::Time, block::height};
    ///
    /// fn foo() {
    ///     let block_height = height();
    ///     let block_time = Time::block(block_height);
    ///     assert(block_time.is_zero() == false);
    /// }
    /// ```
    pub fn block(block_height: u32) -> Self {
        let tia64 = asm(timestamp, height: block_height) {
            time timestamp height;
            timestamp: u64
        };

        Self {
            unix: tia64 - TAI_64_CONVERTER,
        }
    }

    /// Returns the duration of time that has passed since an earlier time.
    ///
    /// # Arguments
    ///
    /// * `earlier`: [Time] - An earlier time to compare to.
    ///
    /// # Returns
    ///
    /// * [Result<Duration, TimeError>] - An `Ok(Duration)` or an `Err(TimeError)` if `earlier` ia later than the `Self` duration.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::{time::{Time, Duration}, block::height};
    ///
    /// fn foo() {
    ///     let now = Time::now();
    ///     let earlier = Time::block(height() - 1u32);
    ///
    ///     let result_duration = now.duration_since(earlier);
    ///     assert(result_duration.is_zero() == false);
    /// }
    /// ```
    pub fn duration_since(self, earlier: Self) -> Result<Duration, TimeError> {
        if self.unix < earlier.unix {
            Err(TimeError::LaterThanTime)
        } else {
            Ok(Duration::seconds(self.unix - earlier.unix))
        }
    }

    /// Returns the duration of time that has passed compared to the current block time.
    ///
    /// # Returns
    ///
    /// * [Result<Duration, TimeError>] - An `Ok(Duration)` or an `Err(TimeError)` if the `Self` duration is after the current block time.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::{time::{Time, Duration}, block::height};
    ///
    /// fn foo() {
    ///     let earlier = Time::block(height() - 2u32);
    ///     let result_duration = earlier.elapsed();
    ///     assert(result_duration.is_zero() == false);
    /// }
    /// ```
    pub fn elapsed(self) -> Result<Duration, TimeError> {
        let tia64 = asm(timestamp, height) {
            bhei height;
            time timestamp height;
            timestamp: u64
        };

        let now = Self {
            unix: tia64 - TAI_64_CONVERTER,
        };

        if self.unix > now.unix {
            Err(TimeError::LaterThanNow)
        } else {
            Ok(Duration::seconds(now.unix - self.unix))
        }
    }

    /// Shifts a `Time` forward by a `Duration`.
    ///
    /// # Arguments
    ///
    /// * `duration`: [Duration] - The amount to increment `Time` by.
    ///
    /// # Returns
    ///
    /// * [Time] - A new UNIX `Time`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::time::{Time, Duration};
    ///
    /// fn foo() {
    ///     let now = Time::now();
    ///     let future = now.add(Duration::DAY);
    ///     assert(future > now);
    /// }
    /// ```
    pub fn add(self, duration: Duration) -> Self {
        Self {
            unix: self.unix + duration.as_seconds(),
        }
    }

    /// Shifts a `Time` backward by a `Duration`.
    ///
    /// # Arguments
    ///
    /// * `duration`: [Duration] - The amount to decrement `Time` by.
    ///
    /// # Returns
    ///
    /// * [Time] - A new UNIX `Time`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::time::{Time, Duration};
    ///
    /// fn foo() {
    ///     let now = Time::now();
    ///     let past = now.subtract(Duration::DAY);
    ///     assert(past < now);
    /// }
    /// ```
    pub fn subtract(self, duration: Duration) -> Self {
        Self {
            unix: self.unix - duration.as_seconds(),
        }
    }

    /// Returns whether a `Time` is zero.
    ///
    /// # Returns
    ///
    /// * [bool] - `true` if the `Time` is zero, otherwise `false`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::time::Time;
    ///
    /// fn foo() {
    ///     let now = Time::now();
    ///     assert(now.is_zero() == false);
    /// }
    /// ```
    pub fn is_zero(self) -> bool {
        self.unix == 0
    }

    /// Creates a new `Time` from a TAI64 timestamp.
    ///
    /// # Arguments
    ///
    /// * `tai64`: [u64] - A TAI64 timestamp.
    ///
    /// # Returns
    ///
    /// * [Time] - A new UNIX `Time`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::{time::Time, block::timestamp};
    ///
    /// fn foo() {
    ///     let tai64_time = timestamp();
    ///     let unix_time = Time::from_tai64(tai64_time);
    ///     assert(unix_time == Time::now());
    /// }
    /// ```
    pub fn from_tai64(tai64: u64) -> Self {
        Self {
            unix: tai64 - TAI_64_CONVERTER,
        }
    }

    /// Returns the UNIX `Time` as TAI64 time.
    ///
    /// # Returns
    ///
    /// * [u64] - The `Time` as TAI64 time.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::{time::Time, block::timestamp};
    ///
    /// fn foo() {
    ///     let unix_time = Time::now();
    ///     let tai64_time = unix_time.as_tai64();
    ///     assert(tai64_time == timestamp());
    /// }
    /// ```
    pub fn as_tai64(self) -> u64 {
        self.unix + TAI_64_CONVERTER
    }

    /// Returns the underlying UNIX timestamp.
    ///
    /// # Returns
    ///
    /// * [u64] - The underlying `u64` UNIX timestamp.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::time::Time;
    ///
    /// fn foo() {
    ///     let now = Time::now();
    ///     let result_u64: u64 = now.unix();
    ///     assert(result_u64 != 0);
    /// }
    /// ```
    pub fn unix(self) -> u64 {
        self.unix
    }
}

impl From<u64> for Time {
    fn from(unix_timestamp: u64) -> Self {
        Self {
            unix: unix_timestamp,
        }
    }
}

impl Into<u64> for Time {
    fn into(self) -> u64 {
        self.unix
    }
}

impl Add<Duration> for Time {
    type Output = Self;
    fn add(self, other: Duration) -> Self {
        self.add(other)
    }
}

impl Subtract<Duration> for Time {
    type Output = Self;
    fn subtract(self, other: Duration) -> Self {
        self.subtract(other)
    }
}

// impl Add for Time {
//     fn add(self, other: Self) -> Self {
//         Self {
//             unix: self.unix + other.unix,
//         }
//     }
// }

// impl Subtract for Time {
//     fn subtract(self, other: Self) -> Self {
//         Self {
//             unix: self.unix - other.unix,
//         }
//     }
// }


impl PartialEq for Time {
    fn eq(self, other: Self) -> bool {
        self.unix == other.unix
    }
}

impl Eq for Time {}

impl Ord for Time {
    fn gt(self, other: Self) -> bool {
        self.unix > other.unix
    }

    fn lt(self, other: Self) -> bool {
        self.unix < other.unix
    }
}

impl OrdEq for Time {}

impl Hash for Time {
    fn hash(self, ref mut state: Hasher) {
        self.unix.hash(state);
    }
}
