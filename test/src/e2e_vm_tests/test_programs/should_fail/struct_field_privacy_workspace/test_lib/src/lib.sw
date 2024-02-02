library;

pub mod lib01;

pub fn use_me() {
    ::lib01::use_me();
    ::lib01::lib01_nested::use_me();
}