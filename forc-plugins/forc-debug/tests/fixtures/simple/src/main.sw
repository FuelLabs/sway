script;

fn main(hi: u64, hi2: u64) -> u64 {
    let h3 = hi + hi2;
    if h3 > 100 {
        return 0;
    }
    h3
}

fn helper(hi: u64, hi2: u64) -> u64 {
    let h3 = hi + hi2;
    if h3 > 100 {
        return 0;
    }
    h3
}

#[test]
fn test_1() {
    let hi = 1;
    let hey = 2;
    let res = helper(hi, hey);
    assert_eq(res, 3);
}

#[test]
fn test_2() {
    let hi = 1;
    let hey = 2;
    let res = helper(hi, hey);
    assert_eq(res, 3);
}

#[test]
fn test_3() {
    let hi = 1;
    let hey = 2;
    let res = helper(hi, hey);
    assert_eq(res, 3);
}
