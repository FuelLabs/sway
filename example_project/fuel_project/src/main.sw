script;
use core::ops::Ord;

struct Rgb {
  red: u64,
  green: u64,
  blue: u64,
}


enum PrimaryColor {
   Red : (),
   Green : (),
   Blue : ()
}

impl Ord for PrimaryColor {
    fn gt(self, other: Self) -> bool {
        asm(r1: self, r2: other, r3) {
            gt r3 r1 r2;
            r3: bool
        }
    }
    fn lt(self, other: Self) -> bool {
        asm(r1: self, r2: other, r3) {
            lt r3 r1 r2;
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
  // TODO: when we support match statements, change this to a match statement
  fn rgb(self) -> Rgb {
    if self == PrimaryColor::Red {
      Rgb {
        red: 255,
        blue: 0,
        green: 0,
      }
    }
    else if self == PrimaryColor::Green {
      Rgb {
        red: 0,
        blue: 0,
        green: 255,
      }
    }
    else if self == PrimaryColor::Blue {
      Rgb {
        red: 0,
        blue: 255,
        green: 0,
      }
    }
    else {
      Rgb {
        red: 0,
        blue: 0,
        green: 0
      }
    }
  }
}

fn main() {
  let first_color = PrimaryColor::Green;
  let rgb: Rgb = first_color.rgb();
}

trait Color {
  fn rgb(self) -> Rgb;
}
