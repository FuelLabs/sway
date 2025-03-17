script;
use std::*;
use std::ops::Ord;

struct Rgb {
    red: u64,
    green: u64,
    blue: u64,
}

trait Color {
    fn rgb(self) -> Rgb;
}

enum PrimaryColor {
    Red: (),
    Green: (),
    Blue: (),
}

impl PartialEq for PrimaryColor {
    fn eq(self, other: Self) -> bool {
        asm(r1: self, r2: other, r3) {
            eq r3 r1 r2;
            r3: bool
        }
    }
}
impl Eq for PrimaryColor {}

impl std::ops::Ord for PrimaryColor {
    fn lt(self, other: Self) -> bool {
        asm(r1: self, r2: other, r3) {
            lt r3 r1 r2;
            r3: bool
        }
    }
    fn gt(self, other: Self) -> bool {
        asm(r1: self, r2: other, r3) {
            gt r3 r1 r2;
            r3: bool
        }
    }
}

impl Color for PrimaryColor {
    // TODO: when we support match statements, change this to a match statement
    fn rgb(self) -> Rgb {
        if self == PrimaryColor::Red {
            Rgb {
                red: 255,
                blue: 0,
                green: 0,
            }
        } else if self == PrimaryColor::Green {
            Rgb {
                red: 0,
                blue: 0,
                green: 255,
            }
        } else if self == PrimaryColor::Blue {
            Rgb {
                red: 0,
                blue: 255,
                green: 0,
            }
        }
        // TODO remove this else when exhaustive ifs are checked for
        else {
            Rgb {
                red: 0,
                green: 0,
                blue: 0,
            }
        }
    }
}

fn main() -> u32 {
    let first_color: PrimaryColor = PrimaryColor::Green;
    let _test = first_color == PrimaryColor::Green;
    // Specifically, when we call methods in the below way, `self` is undefined
    let _rgb: Rgb = first_color.rgb();
    // now, going to test the register pool by using over 48 registers
    let second_color = PrimaryColor::Blue;
    let _second_rgb = second_color.rgb();
    let second_color = PrimaryColor::Blue;
    let _second_rgb = second_color.rgb();
    let second_color = PrimaryColor::Blue;
    let _second_rgb = second_color.rgb();
    10u32
}
