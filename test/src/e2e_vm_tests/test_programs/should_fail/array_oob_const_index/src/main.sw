library;

fn literal_index() -> u64 {
    let ary = [1, 2, 3];
    ary[4]
}

const I: u64 = 4;

fn global_const_index() -> u64 {
    let ary = [1, 2, 3];
    ary[I]
}

#[test]
fn test() {
    let _ = literal_index();
    let _ = global_const_index();
}
