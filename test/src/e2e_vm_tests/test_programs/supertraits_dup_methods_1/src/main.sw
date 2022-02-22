script;

struct Rgb {
    red: u64,
    green: u64,
    blue: u64,
}

trait Color1 {
    fn rgb(self);
}

trait Color2 {
    fn rgb(self);
}

// Color sees both rgb() functions and that's not allowed
trait Color : Color1 + Color2 {

}

enum PrimaryColor {
    Red: (),
    Green: (),
    Blue: (),
}

impl Color for PrimaryColor {
    fn rgb(self) {
    }
}

fn main() -> bool {
   true
}
