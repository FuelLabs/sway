script;

fn main() -> u64 {
    let arr: [u64; 5] = [1, 2, 3, 4, 5];
    let mut idx: u64 = 0;
    let mut total: u64 = 0;
    while idx < 5 {
        total = total + arr[idx as usize];
        idx = idx + 1;
    }
    total
}
