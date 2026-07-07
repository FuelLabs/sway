library;

use std::time::{Duration, Time};
use std::block::{height, timestamp};
use std::flags::{disable_panic_on_overflow, disable_panic_on_unsafe_math};

#[test]
fn time_duration_zero() {
    let zero = Duration::ZERO;
    assert_eq(zero.is_zero(), true);
    assert_eq(zero.as_seconds(), 0);
}

#[test]
fn time_duration_max() {
    let max = Duration::MAX;
    assert_eq(max.as_seconds(), u64::max());
}

#[test]
fn time_duration_min() {
    let min = Duration::MIN;
    assert_eq(min.as_seconds(), u64::min());
}

#[test]
fn time_duration_second() {
    let second = Duration::SECOND;
    assert_eq(second.as_seconds(), 1);
    assert_eq(second.as_minutes(), 0);
    assert_eq(second.as_hours(), 0);
    assert_eq(second.as_days(), 0);
    assert_eq(second.as_weeks(), 0);
}

#[test]
fn time_duration_minute() {
    let minute = Duration::MINUTE;
    assert_eq(minute.as_seconds(), 60);
    assert_eq(minute.as_minutes(), 1);
    assert_eq(minute.as_hours(), 0);
    assert_eq(minute.as_days(), 0);
    assert_eq(minute.as_weeks(), 0);
}

#[test]
fn time_duration_hour() {
    let hour = Duration::HOUR;
    assert_eq(hour.as_seconds(), 3_600);
    assert_eq(hour.as_minutes(), 60);
    assert_eq(hour.as_hours(), 1);
    assert_eq(hour.as_days(), 0);
    assert_eq(hour.as_weeks(), 0);
}

#[test]
fn time_duration_day() {
    let day = Duration::DAY;
    assert_eq(day.as_seconds(), 86_400);
    assert_eq(day.as_minutes(), 1440);
    assert_eq(day.as_hours(), 24);
    assert_eq(day.as_days(), 1);
    assert_eq(day.as_weeks(), 0);
}

#[test]
fn time_duration_week() {
    let day = Duration::WEEK;
    assert_eq(day.as_seconds(), 604_800);
    assert_eq(day.as_minutes(), 10080);
    assert_eq(day.as_hours(), 168);
    assert_eq(day.as_days(), 7);
    assert_eq(day.as_weeks(), 1);
}

#[test]
fn time_duration_seconds() {
    let one_second = Duration::seconds(1);
    let sixty_seconds = Duration::seconds(60);
    let one_twenty_seconds = Duration::seconds(120);
    let week_seconds = Duration::seconds(604_800);

    assert_eq(one_second, Duration::SECOND);
    assert_eq(sixty_seconds, Duration::MINUTE);
    assert_eq(one_twenty_seconds, Duration::minutes(2));
    assert_eq(week_seconds, Duration::WEEK);
}

#[test]
fn time_duration_minutes() {
    let one_minute = Duration::minutes(1);
    let sixty_minutes = Duration::minutes(60);
    let one_twenty_minutes = Duration::minutes(120);
    let week_minutes = Duration::minutes(10080);

    assert_eq(one_minute, Duration::MINUTE);
    assert_eq(sixty_minutes, Duration::HOUR);
    assert_eq(one_twenty_minutes, Duration::hours(2));
    assert_eq(week_minutes, Duration::WEEK);
}

#[test]
fn time_duration_hours() {
    let one_hour = Duration::hours(1);
    let twenty_four_hours = Duration::hours(24);
    let forty_eight_hours = Duration::hours(48);
    let week_hours = Duration::hours(168);

    assert_eq(one_hour, Duration::HOUR);
    assert_eq(twenty_four_hours, Duration::DAY);
    assert_eq(forty_eight_hours, Duration::days(2));
    assert_eq(week_hours, Duration::WEEK);
}

#[test]
fn time_duration_days() {
    let one_day = Duration::days(1);
    let seven_days = Duration::days(7);
    let fourteen_days = Duration::days(14);
    let three_sixty_five_days = Duration::days(364);

    assert_eq(one_day, Duration::DAY);
    assert_eq(seven_days, Duration::WEEK);
    assert_eq(fourteen_days, Duration::weeks(2));
    assert_eq(three_sixty_five_days, Duration::weeks(52));
}

#[test]
fn time_duration_weeks() {
    let one_week = Duration::weeks(1);
    let two_weeks = Duration::weeks(2);
    let fifty_two_weeks = Duration::weeks(52);

    assert_eq(one_week, Duration::WEEK);
    assert_eq(two_weeks, Duration::weeks(2));
    assert_eq(fifty_two_weeks, Duration::days(364));
}

#[test]
fn time_duration_as_seconds() {
    let second = Duration::SECOND;
    let minute = Duration::MINUTE;
    let hour = Duration::HOUR;
    let day = Duration::DAY;
    let week = Duration::WEEK;

    assert_eq(second.as_seconds(), 1);
    assert_eq(minute.as_seconds(), 60);
    assert_eq(hour.as_seconds(), 3_600);
    assert_eq(day.as_seconds(), 86_400);
    assert_eq(week.as_seconds(), 604_800);
}

#[test]
fn time_duration_as_minutes() {
    let second = Duration::SECOND;
    let minute = Duration::MINUTE;
    let hour = Duration::HOUR;
    let day = Duration::DAY;
    let week = Duration::WEEK;

    assert_eq(second.as_minutes(), 0);
    assert_eq(minute.as_minutes(), 1);
    assert_eq(hour.as_minutes(), 60);
    assert_eq(day.as_minutes(), 1_440);
    assert_eq(week.as_minutes(), 10_080);
}

#[test]
fn time_duration_as_hours() {
    let second = Duration::SECOND;
    let minute = Duration::MINUTE;
    let hour = Duration::HOUR;
    let day = Duration::DAY;
    let week = Duration::WEEK;

    assert_eq(second.as_hours(), 0);
    assert_eq(minute.as_hours(), 0);
    assert_eq(hour.as_hours(), 1);
    assert_eq(day.as_hours(), 24);
    assert_eq(week.as_hours(), 168);
}

#[test]
fn time_duration_as_days() {
    let second = Duration::SECOND;
    let minute = Duration::MINUTE;
    let hour = Duration::HOUR;
    let day = Duration::DAY;
    let week = Duration::WEEK;

    assert_eq(second.as_days(), 0);
    assert_eq(minute.as_days(), 0);
    assert_eq(hour.as_days(), 0);
    assert_eq(day.as_days(), 1);
    assert_eq(week.as_days(), 7);
}

#[test]
fn time_duration_as_weeks() {
    let second = Duration::SECOND;
    let minute = Duration::MINUTE;
    let hour = Duration::HOUR;
    let day = Duration::DAY;
    let week = Duration::WEEK;
    let fifty_two_weeks = Duration::weeks(52);

    assert_eq(second.as_weeks(), 0);
    assert_eq(minute.as_weeks(), 0);
    assert_eq(hour.as_weeks(), 0);
    assert_eq(day.as_weeks(), 0);
    assert_eq(week.as_weeks(), 1);
    assert_eq(fifty_two_weeks.as_weeks(), 52);
}

#[test]
fn time_duration_is_zero() {
    let zero = Duration::ZERO;
    let other_zero = Duration::seconds(0);
    let second = Duration::SECOND;
    let minute = Duration::MINUTE;
    let hour = Duration::HOUR;
    let day = Duration::DAY;
    let week = Duration::WEEK;

    assert(zero.is_zero());
    assert(other_zero.is_zero());
    assert(!second.is_zero());
    assert(!minute.is_zero());
    assert(!hour.is_zero());
    assert(!day.is_zero());
    assert(!week.is_zero());
}

#[test]
fn time_duration_add() {
    let zero = Duration::ZERO;
    let second = Duration::SECOND;
    let minute = Duration::MINUTE;
    let hour = Duration::HOUR;
    let day = Duration::DAY;
    let week = Duration::WEEK;

    assert_eq(zero + second, second);
    assert_eq(second + zero, second);
    assert_eq(zero + minute, minute);
    assert_eq(minute + zero, minute);
    assert_eq(zero + hour, hour);
    assert_eq(hour + zero, hour);
    assert_eq(zero + day, day);
    assert_eq(day + zero, day);
    assert_eq(zero + week, week);
    assert_eq(week + zero, week);

    assert_eq(second + second, Duration::seconds(2));
    assert_eq(minute + minute, Duration::minutes(2));
    assert_eq(hour + hour, Duration::hours(2));
    assert_eq(day + day, Duration::days(2));
    assert_eq(week + week, Duration::weeks(2));
    assert_eq(minute + second, Duration::seconds(61));
    assert_eq(second + minute, Duration::seconds(61));

    assert_eq(day + day + day + day + day + day + day, week);
}

#[test]
fn time_duration_overflow_add() {
    let _ = disable_panic_on_overflow();

    let a = Duration::MAX;
    let b = a + Duration::SECOND;

    require(b == Duration::ZERO, b);

    let c = a + Duration::seconds(2);

    require(c == Duration::SECOND, c);

    let d = a + Duration::MAX;

    require(d == Duration::MAX - Duration::SECOND, d);

    let e = a + (Duration::MAX - Duration::SECOND);

    require(e == Duration::MAX - Duration::seconds(2), e);
}

#[test(should_revert)]
fn revert_time_duration_overflow_add() {
    let a = Duration::MAX;
    let b = a + Duration::SECOND;
    log(b);
}

#[test(should_revert)]
fn revert_time_duration_add_unsafe_math() {
    let _ = disable_panic_on_unsafe_math();
    let a = Duration::MAX;
    let b = a + Duration::SECOND;
    log(b);
}

#[test]
fn time_duration_subtract() {
    let zero = Duration::ZERO;
    let second = Duration::SECOND;
    let minute = Duration::MINUTE;
    let hour = Duration::HOUR;
    let day = Duration::DAY;
    let week = Duration::WEEK;

    assert_eq(second - second, zero);
    assert_eq(minute - second, Duration::seconds(59));
    assert_eq(week - day, Duration::days(6));
    assert_eq(zero - zero, zero);
    assert_eq(second - zero, second);
    assert_eq(minute - zero, minute);
    assert_eq(hour - minute, Duration::minutes(59));
}

#[test(should_revert)]
fn revert_time_duration_underflow_sub() {
    let a = Duration::ZERO;
    let b = Duration::SECOND;
    let c = a - b;
    log(c);
}

#[test(should_revert)]
fn revert_time_duration_sub_unsafe_math() {
    let _ = disable_panic_on_unsafe_math();
    let a = Duration::ZERO;
    let b = Duration::SECOND;
    let c = a - b;
    log(c);
}

#[test]
fn time_duration_underflow_sub() {
    let _ = disable_panic_on_overflow();

    let a = Duration::ZERO;
    let b = Duration::SECOND;

    let c = a - b;
    assert_eq(c, Duration::MAX);

    let d = Duration::MAX;

    let e = a - d;
    assert_eq(e, b);
}

#[test]
fn time_duration_eq() {
    let zero_1 = Duration::ZERO;
    let zero_2 = Duration::ZERO;
    let second_1 = Duration::SECOND;
    let second_2 = Duration::SECOND;
    let minute_1 = Duration::MINUTE;
    let minute_2 = Duration::MINUTE;
    let hour_1 = Duration::HOUR;
    let hour_2 = Duration::HOUR;
    let day_1 = Duration::DAY;
    let day_2 = Duration::DAY;
    let week_1 = Duration::WEEK;
    let week_2 = Duration::WEEK;

    assert_eq(zero_1, zero_2);
    assert_eq(second_1, second_2);
    assert_eq(minute_1, minute_2);
    assert_eq(hour_1, hour_2);
    assert_eq(day_1, day_2);
    assert_eq(week_1, week_2);
}

#[test]
fn time_duration_neq() {
    let zero = Duration::ZERO;
    let second = Duration::SECOND;
    let minute = Duration::MINUTE;
    let hour = Duration::HOUR;
    let day = Duration::DAY;
    let week = Duration::WEEK;

    assert_ne(zero, second);
    assert_ne(zero, minute);
    assert_ne(zero, hour);
    assert_ne(zero, day);
    assert_ne(zero, week);

    assert_ne(second, minute);
    assert_ne(second, hour);
    assert_ne(second, day);
    assert_ne(second, week);

    assert_ne(minute, hour);
    assert_ne(minute, day);
    assert_ne(minute, week);

    assert_ne(hour, day);
    assert_ne(hour, week);

    assert_ne(day, week);
}

#[test]
fn time_duration_ord() {
    let zero = Duration::ZERO;
    let second = Duration::SECOND;
    let minute = Duration::MINUTE;
    let hour = Duration::HOUR;
    let day = Duration::DAY;
    let week = Duration::WEEK;

    assert(zero < second);
    assert(zero < minute);
    assert(zero < hour);
    assert(zero < day);
    assert(zero < week);

    assert(second < minute);
    assert(second < hour);
    assert(second < day);
    assert(second < week);

    assert(minute < hour);
    assert(minute < day);
    assert(minute < week);

    assert(hour < day);
    assert(hour < week);

    assert(day < week);

    assert(second > zero);
    assert(minute > zero);
    assert(hour > zero);
    assert(day > zero);
    assert(week > zero);

    assert(minute > second);
    assert(hour > second);
    assert(day > second);
    assert(week > second);

    assert(hour > minute);
    assert(day > minute);
    assert(week > minute);

    assert(day > hour);
    assert(week > hour);

    assert(week > day);
}

#[test]
fn test_duration_time_from_u64() {
    let zero = Duration::ZERO;
    let second = Duration::SECOND;
    let minute = Duration::MINUTE;
    let hour = Duration::HOUR;
    let day = Duration::DAY;
    let week = Duration::WEEK;

    let from_zero = Duration::from(0);
    let from_second = Duration::from(1);
    let from_minute = Duration::from(60);
    let from_hour = Duration::from(3_600);
    let from_day = Duration::from(86_400);
    let from_week = Duration::from(604_800);

    assert_eq(zero, from_zero);
    assert_eq(second, from_second);
    assert_eq(minute, from_minute);
    assert_eq(hour, from_hour);
    assert_eq(day, from_day);
    assert_eq(week, from_week);
}

#[test]
fn test_duration_time_into_u64() {
    let from_zero = Duration::from(0);
    let from_second = Duration::from(1);
    let from_minute = Duration::from(60);
    let from_hour = Duration::from(3_600);
    let from_day = Duration::from(86_400);
    let from_week = Duration::from(604_800);

    assert_eq(from_zero.into(), 0);
    assert_eq(from_second.into(), 1);
    assert_eq(from_minute.into(), 60);
    assert_eq(from_hour.into(), 3_600);
    assert_eq(from_day.into(), 86_400);
    assert_eq(from_week.into(), 604_800);
}

#[test]
fn time_time_new() {
    let new_1 = Time::new(1);
    let new_2 = Time::new(100_000);
    let new_3 = Time::new(100_000_000_000);

    assert_eq(new_1.into(), 1);
    assert_eq(new_2.into(), 100_000);
    assert_eq(new_3.into(), 100_000_000_000);
}

#[test]
fn time_time_duration_since() {
    let time_1 = Time::new(100_000);
    let time_2 = Time::new(200_000);
    let time_3 = Time::new(300_000);
    let time_4 = Time::new(400_000);

    let duration_1 = time_2.duration_since(time_1).unwrap();
    assert_eq(duration_1.as_seconds(), 100_000);

    let duration_2 = time_3.duration_since(time_1).unwrap();
    assert_eq(duration_2.as_seconds(), 200_000);

    let duration_3 = time_4.duration_since(time_1).unwrap();
    assert_eq(duration_3.as_seconds(), 300_000);

    let duration_4 = time_1.duration_since(time_1).unwrap();
    assert(duration_4.is_zero());

    let duration_5 = time_1.duration_since(time_2);
    assert(duration_5.is_err());
}

#[test]
fn time_time_add() {
    let time_1 = Time::new(100_000);
    let time_2 = Time::new(200_000);
    let time_3 = Time::new(300_000);
    let time_4 = Time::new(400_000);

    let duration_1 = Duration::seconds(100_000);
    let duration_2 = Duration::seconds(200_000);
    let duration_3 = Duration::seconds(300_000);
    let duration_4 = Duration::ZERO;

    assert_eq(time_1.add(duration_1), time_2);
    // assert_eq(duration_1 + time_1, time_2);
    assert_eq(time_1.add(duration_2), time_3);
    // assert_eq(duration_2 + time_1, time_3);
    assert_eq(time_1.add(duration_3), time_4);
    // assert_eq(duration_3 + time_1, time_4);
    assert_eq(time_1.add(duration_4), time_1);
    // assert_eq(duration_4 + time_1, time_1);

    assert_eq(time_2.add(duration_1), time_3);
    assert_eq(time_2.add(duration_2), time_4);
    assert_eq(time_2.add(duration_4), time_2);

    assert_eq(time_3.add(duration_1), time_4);
    assert_eq(time_3.add(duration_4), time_3);

    assert_eq(time_4.add(duration_4), time_4);
}

#[test]
fn time_time_overflow_add() {
    let _ = disable_panic_on_overflow();

    let a = Time::new(u64::max());
    let b = a.add(Duration::SECOND);

    require(b == Time::new(0), b);

    let c = a.add(Duration::seconds(2));

    require(c == Time::new(1), c);

    let d = a.add(Duration::MAX);

    require(d == Time::new(u64::max()).subtract(Duration::SECOND), d);

    let e = a.add(Duration::MAX - Duration::SECOND);

    require(e == Time::new(u64::max() - 2), e);
}

#[test(should_revert)]
fn revert_time_time_overflow_add() {
    let a = Time::new(u64::max());
    let b = a.add(Duration::SECOND);
    log(b);
}

#[test(should_revert)]
fn revert_time_time_add_unsafe_math() {
    let _ = disable_panic_on_unsafe_math();
    let a = Time::new(u64::max());
    let b = a.add(Duration::SECOND);
    log(b);
}

#[test]
fn time_time_subtract() {
    let time_1 = Time::new(100_000);
    let time_2 = Time::new(200_000);
    let time_3 = Time::new(300_000);
    let time_4 = Time::new(400_000);

    let duration_1 = Duration::seconds(100_000);
    let duration_2 = Duration::seconds(200_000);
    let duration_3 = Duration::seconds(300_000);
    let duration_4 = Duration::ZERO;

    assert_eq(time_1.subtract(duration_1), Time::new(0));
    assert_eq(time_2.subtract(duration_2), Time::new(0));
    assert_eq(time_3.subtract(duration_3), Time::new(0));

    assert_eq(time_1.subtract(duration_4), time_1);
    assert_eq(time_2.subtract(duration_4), time_2);
    assert_eq(time_3.subtract(duration_4), time_3);
    assert_eq(time_4.subtract(duration_4), time_4);

    assert_eq(time_2.subtract(duration_1), time_1);
    assert_eq(time_3.subtract(duration_1), time_2);
    assert_eq(time_4.subtract(duration_1), time_3);

    assert_eq(time_3.subtract(duration_2), time_1);
    assert_eq(time_4.subtract(duration_2), time_2);

    assert_eq(time_4.subtract(duration_3), time_1);
}

#[test(should_revert)]
fn revert_time_time_underflow_sub() {
    let a = Time::new(0);
    let b = Duration::SECOND;
    let c = a.subtract(b);
    log(c);
}

#[test(should_revert)]
fn revert_time_time_sub_unsafe_math() {
    let _ = disable_panic_on_unsafe_math();
    let a = Time::new(0);
    let b = Duration::SECOND;
    let c = a.subtract(b);
    log(c);
}

#[test]
fn time_time_underflow_sub() {
    let _ = disable_panic_on_overflow();

    let a = Time::new(0);
    let b = Duration::SECOND;

    let c = a.subtract(b);
    assert_eq(c, Time::new(u64::max()));

    let d = Duration::MAX;

    let e = a.subtract(d);
    assert_eq(e, Time::new(1));
}

#[test]
fn time_time_is_zero() {
    let zero = Time::new(0);
    let not_zero = Time::new(1);
    let max = Time::new(u64::max());

    assert(zero.is_zero());
    assert(!not_zero.is_zero());
    assert(!max.is_zero());
}

#[test]
fn time_time_from_u64() {
    let from_1 = Time::new(0);
    let from_2 = Time::new(1);
    let from_3 = Time::new(u64::max());

    assert_eq(from_1.unix(), 0);
    assert_eq(from_2.unix(), 1);
    assert_eq(from_3.unix(), u64::max());
}

#[test]
fn time_time_into_u64() {
    let from_1 = Time::new(0);
    let from_2 = Time::new(1);
    let from_3 = Time::new(u64::max());

    assert_eq(from_1.into(), 0);
    assert_eq(from_2.into(), 1);
    assert_eq(from_3.into(), u64::max());
}

#[test]
fn time_time_eq() {
    let time_1 = Time::new(100_000);
    let time_2 = Time::new(100_000);
    let time_3 = Time::new(200_000);
    let time_4 = Time::new(200_000);
    let time_5 = Time::new(300_000);
    let time_6 = Time::new(300_000);
    let time_7 = Time::new(400_000);
    let time_8 = Time::new(400_000);
    let time_9 = Time::new(0);
    let time_10 = Time::new(0);
    let time_11 = Time::new(u64::max());
    let time_12 = Time::new(u64::max());

    assert_eq(time_1, time_2);
    assert_eq(time_3, time_4);
    assert_eq(time_5, time_6);
    assert_eq(time_7, time_8);
    assert_eq(time_9, time_10);
    assert_eq(time_11, time_12);
}

#[test]
fn time_time_neq() {
    let time_1 = Time::new(100_000);
    let time_2 = Time::new(200_000);
    let time_3 = Time::new(300_000);
    let time_4 = Time::new(400_000);
    let time_5 = Time::new(0);
    let time_6 = Time::new(u64::max());

    assert_ne(time_1, time_2);
    assert_ne(time_1, time_3);
    assert_ne(time_1, time_4);
    assert_ne(time_1, time_5);
    assert_ne(time_1, time_6);

    assert_ne(time_2, time_3);
    assert_ne(time_2, time_4);
    assert_ne(time_2, time_5);
    assert_ne(time_2, time_6);

    assert_ne(time_3, time_4);
    assert_ne(time_3, time_5);
    assert_ne(time_3, time_6);

    assert_ne(time_4, time_5);
    assert_ne(time_4, time_6);

    assert_ne(time_5, time_6);
}

#[test]
fn time_time_ord() {
    let time_1 = Time::new(0);
    let time_2 = Time::new(100_000);
    let time_3 = Time::new(200_000);
    let time_4 = Time::new(300_000);
    let time_5 = Time::new(400_000);
    let time_6 = Time::new(u64::max());

    assert(time_1 < time_2);
    assert(time_1 < time_3);
    assert(time_1 < time_4);
    assert(time_1 < time_5);
    assert(time_1 < time_6);

    assert(time_2 > time_1);
    assert(time_3 > time_1);
    assert(time_4 > time_1);
    assert(time_5 > time_1);
    assert(time_6 > time_1);

    assert(time_2 < time_3);
    assert(time_2 < time_4);
    assert(time_2 < time_5);
    assert(time_2 < time_6);

    assert(time_3 > time_2);
    assert(time_4 > time_2);
    assert(time_5 > time_2);
    assert(time_6 > time_2);

    assert(time_3 < time_4);
    assert(time_3 < time_5);
    assert(time_3 < time_6);

    assert(time_4 > time_3);
    assert(time_5 > time_3);
    assert(time_6 > time_3);

    assert(time_4 < time_5);
    assert(time_4 < time_6);

    assert(time_5 > time_4);
    assert(time_6 > time_4);

    assert(time_5 < time_6);

    assert(time_6 > time_5);
}

#[test]
fn time_time_now() {
    // `Time::now()` is derived from the current block's TAI64 timestamp,
    // so converting it back to TAI64 must equal `timestamp()`.
    let now = Time::now();
    assert_eq(now.as_tai64(), timestamp());
}

#[test]
fn time_time_block() {
    // The time of the current block equals `Time::now()`.
    assert_eq(Time::block(height()), Time::now());
}

#[test]
fn time_time_from_tai64() {
    // Constructing from the current block's TAI64 timestamp equals `now()`.
    assert_eq(Time::from_tai64(timestamp()), Time::now());
}

#[test]
fn time_time_as_tai64() {
    // `as_tai64` and `from_tai64` round-trip.
    let time_1 = Time::new(0);
    let time_2 = Time::new(100_000);
    let time_3 = Time::new(u64::max() - Time::new(0).as_tai64());

    assert_eq(Time::from_tai64(time_1.as_tai64()), time_1);
    assert_eq(Time::from_tai64(time_2.as_tai64()), time_2);
    assert_eq(Time::from_tai64(time_3.as_tai64()), time_3);
}
