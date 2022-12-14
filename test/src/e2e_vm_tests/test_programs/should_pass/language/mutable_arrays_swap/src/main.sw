script;

fn main() -> u64 {
    let mut my_array_0: [u64; 1] = [1];
    my_array_0[0] = 10;

    let mut my_array_1: [u64; 1] = [1];
    my_array_1[0] = 20;

    my_array_0[0] = my_array_1[0];
    my_array_0[0]
}
