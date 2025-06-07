library;

use std::time::*;

// ANCHOR: create_durations
fn create_durations() {
    // Using constants
    let zero = Duration::ZERO;
    let second = Duration::SECOND;
    let minute = Duration::MINUTE;
    let hour = Duration::HOUR;
    let day = Duration::DAY;
    let week = Duration::WEEK;

    // Using constructor methods
    let thirty_seconds = Duration::seconds(30);
    let two_hours = Duration::hours(2);
    let three_days = Duration::days(3);
}
// ANCHOR_END: create_durations

// ANCHOR: convert_durations
fn convert_durations() {
    let two_days = Duration::days(2);

    assert(two_days.as_seconds() == 172800); // 2 * 86400
    assert(two_days.as_minutes() == 2880); // 2 * 1440
    assert(two_days.as_hours() == 48); // 2 * 24
    assert(two_days.as_days() == 2);
    assert(two_days.as_weeks() == 0); // Truncated value
}
// ANCHOR_END: convert_durations

// ANCHOR: duration_operations
fn duration_operations() {
    let day1 = Duration::DAY;
    let day2 = Duration::days(1);

    // Equality
    assert(day1 == day2);

    // Addition
    let two_days = day1 + day2;
    assert(two_days.as_days() == 2);

    // Subtraction
    let half_day = two_days - Duration::days(1).add(Duration::hours(12));
    assert(half_day.as_hours() == 12);

    // Comparison
    assert(Duration::MINUTE < Duration::HOUR);
}
// ANCHOR_END: duration_operations

// ANCHOR: create_timestamps
fn create_timestamps() {
    // Current block time
    let now = Time::now();

    // Specific block time
    let block_time = Time::block(12345);

    // From UNIX timestamp
    let custom_time = Time::new(1672531200); // Jan 1, 2023 00:00:00 UTC
}
// ANCHOR_END: create_timestamps

// ANCHOR: time_operations
fn time_operations() {
    let now = Time::now();
    let yesterday = now.subtract(Duration::DAY);
    let tomorrow = now.add(Duration::DAY);

    // Duration calculations
    let elapsed = now.duration_since(yesterday).unwrap();
    assert(elapsed.as_days() == 1);

    // Comparison
    assert(yesterday < now);
    assert(tomorrow > now);
}
// ANCHOR_END: time_operations

// ANCHOR: tai64_conversion
fn tai64_conversion() {
    let now = Time::now();

    // Convert to TAI64
    let tai64 = now.as_tai64();

    // Convert back to UNIX time
    let converted = Time::from_tai64(tai64);
    assert(now == converted);
}
// ANCHOR_END: tai64_conversion
