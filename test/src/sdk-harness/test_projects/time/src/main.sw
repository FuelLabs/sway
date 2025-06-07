contract;

use std::time::Time;
use std::block::{timestamp, height};

abi TimeTest {
    fn get_now() -> Time;
    fn get_block(block: u32) -> Time;
    fn get_tai64() -> u64;
    fn get_height_and_time() -> (u32, Time);
    fn get_time_and_tia64() -> (Time, u64);
    fn from_tia64(tai64: u64) -> Time;
    fn into_tai64(time: Time) -> u64;
}

impl TimeTest for Contract {
    fn get_now() -> Time {
        Time::now()
    }
    fn get_block(block: u32) -> Time {
        Time::block(block)
    }
    fn get_tai64() -> u64 {
        timestamp()
    }
    fn get_height_and_time() -> (u32, Time) {
        (height(), Time::now())
    }
    fn get_time_and_tia64() -> (Time, u64) {
        (Time::now(), timestamp())
    }
    fn from_tia64(tai64: u64) -> Time {
        Time::from_tai64(tai64)
    }
    fn into_tai64(time: Time) -> u64 {
        time.as_tai64()
    }
}