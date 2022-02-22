script;

struct Rgb {
    red: u64,
    green: u64,
    blue: u64,
}

trait Color1 {
    fn rgb1(self);
} {
    fn rgb_wrapper(self) {
        self.rgb1();
    }
}

trait Color2 {
    fn rgb2(self);
} {
    fn rgb_wrapper(self) {
        self.rgb2();
    }
}


// Color sees both rgb_wrapper() functions and that's not allowed
trait Color : Color1 + Color2 {

}

enum PrimaryColor {
    Red: (),
    Green: (),
    Blue: (),
}

impl Color for PrimaryColor {
    fn rgb1(self) {
    }

    fn rgb2(self) {
    }
}

fn main() -> bool {
   true
}
