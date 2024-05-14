script;

fn main() {
    let mut array = [1u64, 2, 3];

    let r_array_1 = &array;

    *r_array_1 = [2, 3, 4];


    let r_array_2 = r_array_1;

    *r_array_2 = [2, 3, 4];


    let r_array_3 = &get_array(
        1, 2
    );

    *r_array_3 = [2, 3, 4];

    *&array = [2, 3, 4];
}

fn get_array(_x: u64, _y: u64) -> [u64;3] {
    [0, 0, 0]
}