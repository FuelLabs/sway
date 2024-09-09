script;

fn main() {
    let mut x = 123;

    let _: &u8 = &x; // No error here.

    let _: &mut u8 = &mut x; // No error here.

    let _: &u8 = &mut x; // No error here.

    let _: &mut u8 = &x;

    let _: &mut &mut &mut u8 = &mut &mut x;

    let _: &mut &mut u8 = &mut &mut &mut x;
}

