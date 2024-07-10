script;

fn main()  {
    let array: [u64; 4] = [1, 2, 3, 4];
    let slice: __slice[u64] = __slice(array, 0, 4);
}
