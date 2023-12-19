contract;

struct S {
    i: u64,
    j: u64,
}


abi TestContract {
    fn foo(a: [u64; 10]) -> u64;
    fn bar(s: S) -> u64;
    fn boo(s: [S; 4]) -> u64;
}

impl TestContract for Contract {
     fn foo(a: [u64; 10]) -> u64 {
        a[9]
     }
     fn bar(s: S) -> u64 {
        s.j
     }
     fn boo(sa: [S; 4]) -> u64 {
        sa[2].j
     }
}

#[test]
fn test1() {
    let caller = abi(TestContract, CONTRACT_ID);

    let a = [0,1,2,3,4,5,6,7,8,9];
    assert(caller.foo(a) == 9);

    let s = S { i : 0, j : 108 };
    assert(caller.bar(s) == 108);

    let sa = [S { i: 0, j: 101 }, S { i: 1, j: 102 }, S { i: 2, j: 103 }, S { i: 3, j: 104 }];
    assert(caller.boo(sa) == 103);
}
