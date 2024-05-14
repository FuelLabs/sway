script;

struct S {
    x: u8,
    u8_field: u8,
}

fn main() {
    let mut not_array = 0;
    let _ = not_array[0];
    not_array[0] = 1;

    let mut s = S { x: 0, u8_field: 0 };
    let _ = s[0];
    s[0] = 1;

    let _ = s.x[0];
    s.x[0] = 1;

    let mut array = [s, s];
    let _ = array[0].x[0];
    array[0].x[0] = 1;

    let _= array[0].u8_field[0];
    array[0].u8_field[0] = 1;

    let _ = array[0][0];
    array[0][0] = 1;

    let mut tuple = (1, 2);
    let _ = tuple[0];
    tuple[0] = 1;

    let _ = tuple.1[0];
    tuple.1[0] = 1;
}
