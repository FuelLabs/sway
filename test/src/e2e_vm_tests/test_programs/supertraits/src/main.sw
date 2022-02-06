script;
use core::*;
use core::ops::Ord;

struct Rgb {
    red: u64,
    green: u64,
    blue: u64,
}

trait ColorBase1 {
    fn rgb1(self) -> Rgb;
} {
    fn rgb1_wrapper(self) -> Rgb {
        self.rgb1()
    } 
} 

trait ColorBase2 {
    fn rgb2(self) -> Rgb;
} {
    fn rgb2_wrapper(self) -> Rgb {
        self.rgb2()
    } 
} 


trait ColorBase3 : ColorBase1 {}

trait ColorBase4 : ColorBase2 {}

trait Color : ColorBase3 + ColorBase4 {
    fn rgb(self) -> Rgb;
} {
    fn rgb_wrapper(self) -> Rgb {
        self.rgb()
    } 
} 

enum PrimaryColor {
    Red: (),
    Green: (),
    Blue: (),
}

impl core::ops::Ord for PrimaryColor {
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
    fn eq(self, other: Self) -> bool {
        asm(r1: self, r2: other, r3) {
            eq r3 r1 r2;
            r3: bool
        }
    }
}

impl Color for PrimaryColor {
    // Method from Color
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
        else {
            Rgb {
                red: 0,
                green: 0,
                blue: 0,
            }
        }
    }   

    // Method from ColorBase1
    fn rgb1(self) -> Rgb {
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
        else {
            Rgb {
                red: 0,
                green: 0,
                blue: 0,
            }
        }
    }

    // Method from ColorBase2
    fn rgb2(self) -> Rgb {
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
    // Test all methods from Color1
    let first_color_1: PrimaryColor = PrimaryColor::Green;
    let first_rgb_1: Rgb = first_color_1.rgb();
    let first_color_2: PrimaryColor = PrimaryColor::Green;
    let first_rgb_2: Rgb = first_color_2.rgb_wrapper();

    // Test all methods from Color2
    let second_color_1: PrimaryColor = PrimaryColor::Blue;
    let second_rgb_1: Rgb = second_color_1.rgb1();
    let second_color_2: PrimaryColor = PrimaryColor::Blue;
    let second_rgb_2: Rgb = second_color_2.rgb1_wrapper();

    // Test all methods from Color
    let third_color_1: PrimaryColor = PrimaryColor::Red;
    let third_rgb_1: Rgb = third_color_1.rgb2();
    let third_color_2: PrimaryColor = PrimaryColor::Red;
    let third_rgb_2: Rgb = third_color_2.rgb2_wrapper();

   10u32
}
