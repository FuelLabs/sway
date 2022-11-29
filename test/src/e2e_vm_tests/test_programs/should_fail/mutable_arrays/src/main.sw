script;

fn main() -> bool {
    let mut b = false;
    b[0] = true;

    let my_array: [u64; 1] = [1];
    my_array[0] = 0;

    takes_ref_mut_arr(my_array);

    let mut my_array_2: [u64; 1] = [1];
    my_array_2[0] = false;
    my_array_2[0][1] = false;

    false
}

fn takes_ref_mut_arr(ref mut arr: [u64; 1]) {

}
