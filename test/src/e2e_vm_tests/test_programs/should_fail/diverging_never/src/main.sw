script;

fn main() {
    let _b = {
        return 5;
    }[0];

    let _: ! = 123u8;  // ERROR.
}
