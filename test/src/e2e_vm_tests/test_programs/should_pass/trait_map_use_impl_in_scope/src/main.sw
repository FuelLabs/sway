script;

use std::time::Time;
// use std::time::*;

pub fn main() {
    let duration = Time::now().duration_since(Time::from(0)).unwrap();
    let _ = duration + duration;
}
