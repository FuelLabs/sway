script;

fn main() -> u64 {
    let mut arr: [u64; 1] = [1];
    takes_ref_mut_array(arr);
    arr[0]
}

#[inline(always)]
fn takes_ref_mut_array(ref mut arr: [u64; 1]) {
    arr[0] = 10;
}
