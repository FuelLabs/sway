library;

use std::time::{Time, Duration};
use std::flags::{disable_panic_on_overflow, disable_panic_on_unsafe_math};

#[test]
fn time_duration_zero() {
    let zero = Duration::ZERO;
    assert(zero.is_zero() == true);
    assert(zero.as_seconds() == 0);
}

#[test]
fn time_duration_max() {
    let max = Duration::MAX;
    assert(max.as_seconds() == u64::max());
}

#[test]
fn time_duration_min() {
    let min = Duration::MIN;
    assert(min.as_seconds() == u64::min());
}

#[test]
fn time_duration_second() {
    let second = Duration::SECOND;
    assert(second.as_seconds() == 1);
    assert(second.as_minutes() == 0);
    assert(second.as_hours() == 0);
    assert(second.as_days() == 0);
    assert(second.as_weeks() == 0);
}

#[test]
fn time_duration_minute() {
    let minute = Duration::MINUTE;
    assert(minute.as_seconds() == 60);
    assert(minute.as_minutes() == 1);
    assert(minute.as_hours() == 0);
    assert(minute.as_days() == 0);
    assert(minute.as_weeks() == 0);
}

#[test]
fn time_duration_hour() {
    let hour = Duration::HOUR;
    assert(hour.as_seconds() == 3_600);
    assert(hour.as_minutes() == 60);
    assert(hour.as_hours() == 1);
    assert(hour.as_days() == 0);
    assert(hour.as_weeks() == 0);
}

#[test]
fn time_duration_day() {
    let day = Duration::DAY;
    assert(day.as_seconds() == 86_400);
    assert(day.as_minutes() == 1440);
    assert(day.as_hours() == 24);
    assert(day.as_days() == 1);
    assert(day.as_weeks() == 0);
}

#[test]
fn time_duration_week() {
    let day = Duration::WEEK;
    assert(day.as_seconds() == 604_800);
    assert(day.as_minutes() == 10080);
    assert(day.as_hours() == 168);
    assert(day.as_days() == 7);
    assert(day.as_weeks() == 1);
}

#[test]
fn time_duration_seconds() {
    let one_second = Duration::seconds(1);
    let sixty_seconds = Duration::seconds(60);
    let one_twenty_seconds = Duration::seconds(120);
    let week_seconds = Duration::seconds(604_800);

    assert(one_second == Duration::SECOND);
    assert(sixty_seconds == Duration::MINUTE);
    assert(one_twenty_seconds == Duration::minutes(2));
    assert(week_seconds == Duration::WEEK);
}

#[test]
fn time_duration_minutes() {
    let one_minute = Duration::minutes(1);
    let sixty_minutes = Duration::minutes(60);
    let one_twenty_minutes = Duration::minutes(120);
    let week_minutes = Duration::minutes(10080);

    assert(one_minute == Duration::MINUTE);
    assert(sixty_minutes == Duration::HOUR);
    assert(one_twenty_minutes == Duration::hours(2));
    assert(week_minutes == Duration::WEEK);
}

#[test]
fn time_duration_hours() {
    let one_hour = Duration::hours(1);
    let twenty_four_hours = Duration::hours(24);
    let forty_eight_hours = Duration::hours(48);
    let week_hours = Duration::hours(168);

    assert(one_hour == Duration::HOUR);
    assert(twenty_four_hours == Duration::DAY);
    assert(forty_eight_hours == Duration::days(2));
    assert(week_hours == Duration::WEEK);
}

#[test]
fn time_duration_days() {
    let one_day = Duration::days(1);
    let seven_days = Duration::days(7);
    let fourteen_days = Duration::days(14);
    let three_sixty_five_days = Duration::days(364);

    assert(one_day == Duration::DAY);
    assert(seven_days == Duration::WEEK);
    assert(fourteen_days == Duration::weeks(2));
    assert(three_sixty_five_days == Duration::weeks(52));
}

#[test]
fn time_duration_weeks() {
    let one_week = Duration::weeks(1);
    let two_weeks = Duration::weeks(2);
    let fifty_two_weeks = Duration::weeks(52);

    assert(one_week == Duration::WEEK);
    assert(two_weeks == Duration::weeks(2));
    assert(fifty_two_weeks == Duration::days(364));
}

#[test]
fn time_duration_as_seconds() {
    let second = Duration::SECOND;
    let minute = Duration::MINUTE;
    let hour = Duration::HOUR;
    let day = Duration::DAY;
    let week = Duration::WEEK;

    assert(second.as_seconds() == 1);
    assert(minute.as_seconds() == 60);
    assert(hour.as_seconds() == 3_600);
    assert(day.as_seconds() == 86_400);
    assert(week.as_seconds() == 604_800);
}

#[test]
fn time_duration_as_minutes() {
    let second = Duration::SECOND;
    let minute = Duration::MINUTE;
    let hour = Duration::HOUR;
    let day = Duration::DAY;
    let week = Duration::WEEK;

    assert(second.as_minutes() == 0);
    assert(minute.as_minutes() == 1);
    assert(hour.as_minutes() == 60);
    assert(day.as_minutes() == 1_440);
    assert(week.as_minutes() == 10_080);
}

#[test]
fn time_duration_as_hours() {
    let second = Duration::SECOND;
    let minute = Duration::MINUTE;
    let hour = Duration::HOUR;
    let day = Duration::DAY;
    let week = Duration::WEEK;

    assert(second.as_hours() == 0);
    assert(minute.as_hours() == 0);
    assert(hour.as_hours() == 1);
    assert(day.as_hours() == 24);
    assert(week.as_hours() == 168);
}

#[test]
fn time_duration_as_days() {
    let second = Duration::SECOND;
    let minute = Duration::MINUTE;
    let hour = Duration::HOUR;
    let day = Duration::DAY;
    let week = Duration::WEEK;

    assert(second.as_days() == 0);
    assert(minute.as_days() == 0);
    assert(hour.as_days() == 0);
    assert(day.as_days() == 1);
    assert(week.as_days() == 7);
}

#[test]
fn time_duration_as_weeks() {
    let second = Duration::SECOND;
    let minute = Duration::MINUTE;
    let hour = Duration::HOUR;
    let day = Duration::DAY;
    let week = Duration::WEEK;
    let fifty_two_weeks = Duration::weeks(52);

    assert(second.as_weeks() == 0);
    assert(minute.as_weeks() == 0);
    assert(hour.as_weeks() == 0);
    assert(day.as_weeks() == 0);
    assert(week.as_weeks() == 1);
    assert(fifty_two_weeks.as_weeks() == 52);
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

    assert(zero + second == second);
    assert(second + zero == second);
    assert(zero + minute == minute);
    assert(minute + zero == minute);
    assert(zero + hour == hour);
    assert(hour + zero == hour);
    assert(zero + day == day);
    assert(day + zero == day);
    assert(zero + week == week);
    assert(week + zero == week);

    assert(second + second == Duration::seconds(2));
    assert(minute + minute == Duration::minutes(2));
    assert(hour + hour == Duration::hours(2));
    assert(day + day == Duration::days(2));
    assert(week + week == Duration::weeks(2));
    assert(minute + second == Duration::seconds(61));
    assert(second + minute == Duration::seconds(61));

    assert(day + day + day + day + day + day + day == week);
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

    assert(second - second == zero);
    assert(minute - second == Duration::seconds(59));
    assert(week - day == Duration::days(6));
    assert(zero - zero == zero);
    assert(second - zero == second);
    assert(minute - zero == minute);
    assert(hour - minute == Duration::minutes(59));
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
    assert(c == Duration::MAX);

    let d = Duration::MAX;

    let e = a - d;
    assert(e == b);
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

    assert(zero_1 == zero_2);
    assert(second_1 == second_2);
    assert(minute_1 == minute_2);
    assert(hour_1 == hour_2);
    assert(day_1 == day_2);
    assert(week_1 == week_2);
}

#[test]
fn time_duration_neq() {
    let zero = Duration::ZERO;
    let second = Duration::SECOND;
    let minute = Duration::MINUTE;
    let hour = Duration::HOUR;
    let day = Duration::DAY;
    let week = Duration::WEEK;

    assert(zero != second);
    assert(zero != minute);
    assert(zero != hour);
    assert(zero != day);
    assert(zero != week);

    assert(second != minute);
    assert(second != hour);
    assert(second != day);
    assert(second != week);

    assert(minute != hour);
    assert(minute != day);
    assert(minute != week);

    assert(hour != day);
    assert(hour != week);

    assert(day != week);
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

    assert(zero == from_zero);
    assert(second == from_second);
    assert(minute == from_minute);
    assert(hour == from_hour);
    assert(day == from_day);
    assert(week == from_week);
}

#[test]
fn test_duration_time_into_u64() {
    let from_zero = Duration::from(0);
    let from_second = Duration::from(1);
    let from_minute = Duration::from(60);
    let from_hour = Duration::from(3_600);
    let from_day = Duration::from(86_400);
    let from_week = Duration::from(604_800);

    assert(from_zero.into() == 0);
    assert(from_second.into() == 1);
    assert(from_minute.into() == 60);
    assert(from_hour.into() == 3_600);
    assert(from_day.into() == 86_400);
    assert(from_week.into() == 604_800);
}

#[test]
fn time_time_new() {
    let new_1 = Time::new(1);
    let new_2 = Time::new(100_000);
    let new_3 = Time::new(100_000_000_000);

    assert(new_1.into() == 1);
    assert(new_2.into() == 100_000);
    assert(new_3.into() == 100_000_000_000);
}

#[test]
fn time_time_duration_since() {
    let time_1 = Time::new(100_000);
    let time_2 = Time::new(200_000);
    let time_3 = Time::new(300_000);
    let time_4 = Time::new(400_000);

    let duration_1 = time_2.duration_since(time_1).unwrap();
    assert(duration_1.as_seconds() == 100_000);

    let duration_2 = time_3.duration_since(time_1).unwrap();
    assert(duration_2.as_seconds() == 200_000);

    let duration_3 = time_4.duration_since(time_1).unwrap();
    assert(duration_3.as_seconds() == 300_000);

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

    assert(time_1.add(duration_1) == time_2);
    // assert(duration_1 + time_1 == time_2);
    assert(time_1.add(duration_2) == time_3);
    // assert(duration_2 + time_1 == time_3);
    assert(time_1.add(duration_3) == time_4);
    // assert(duration_3 + time_1 == time_4);
    assert(time_1.add(duration_4) == time_1);
    // assert(duration_4 + time_1 == time_1);

    assert(time_2.add(duration_1) == time_3);
    assert(time_2.add(duration_2) == time_4);
    assert(time_2.add(duration_4) == time_2);

    assert(time_3.add(duration_1) == time_4);
    assert(time_3.add(duration_4) == time_3);

    assert(time_4.add(duration_4) == time_4);
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

    assert(time_1.subtract(duration_1) == Time::new(0));
    assert(time_2.subtract(duration_2) == Time::new(0));
    assert(time_3.subtract(duration_3) == Time::new(0));
    
    assert(time_1.subtract(duration_4) == time_1);
    assert(time_2.subtract(duration_4) == time_2);
    assert(time_3.subtract(duration_4) == time_3);
    assert(time_4.subtract(duration_4) == time_4);

    assert(time_2.subtract(duration_1) == time_1);
    assert(time_3.subtract(duration_1) == time_2);
    assert(time_4.subtract(duration_1) == time_3);

    assert(time_3.subtract(duration_2) == time_1);
    assert(time_4.subtract(duration_2) == time_2);

    assert(time_4.subtract(duration_3) == time_1);
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
    assert(c == Time::new(u64::max()));

    let d = Duration::MAX;

    let e = a.subtract(d);
    assert(e == Time::new(1));
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

    assert(from_1.unix() == 0);
    assert(from_2.unix() == 1);
    assert(from_3.unix() == u64::max());
}

#[test]
fn time_time_into_u64() {
    let from_1 = Time::new(0);
    let from_2 = Time::new(1);
    let from_3 = Time::new(u64::max());

    assert(from_1.into() == 0);
    assert(from_2.into() == 1);
    assert(from_3.into() == u64::max());
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

    assert(time_1 == time_2);
    assert(time_3 == time_4);
    assert(time_5 == time_6);
    assert(time_7 == time_8);
    assert(time_9 == time_10);
    assert(time_11 == time_12);
}

#[test]
fn time_time_neq() {
    let time_1 = Time::new(100_000);
    let time_2 = Time::new(200_000);
    let time_3 = Time::new(300_000);
    let time_4 = Time::new(400_000);
    let time_5 = Time::new(0);
    let time_6 = Time::new(u64::max());

    assert(time_1 != time_2);
    assert(time_1 != time_3);
    assert(time_1 != time_4);
    assert(time_1 != time_5);
    assert(time_1 != time_6);

    assert(time_2 != time_3);
    assert(time_2 != time_4);
    assert(time_2 != time_5);
    assert(time_2 != time_6);

    assert(time_3 != time_4);
    assert(time_3 != time_5);
    assert(time_3 != time_6);

    assert(time_4 != time_5);
    assert(time_4 != time_6);

    assert(time_5 != time_6);
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
