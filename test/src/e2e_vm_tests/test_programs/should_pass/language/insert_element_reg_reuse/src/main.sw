script;

impl PartialEq for [u64; 2] {
    fn eq(self, other: Self) -> bool {
        let mut i = 0;
        while i < 2 {
            if self[i] != other[i] {
                return false;
            }
            i += 1;
        }
        true
    }
}

impl Eq for [u64; 2] {}

fn tester1(arg: Vec<[u64; 2]>) {
    let mut expected = Vec::new();
    expected.push([0, 1]);
    expected.push([0, 1]);

    assert(arg == expected);
}

fn tester2(arg: Vec<[u64; 2]>) {
    let mut expected = Vec::new();
    expected.push([0, 1]);
    expected.push([0, 1]);

    assert(arg != expected);
}

fn main() -> u64 {
    let mut arg1 = Vec::new();
    arg1.push([0, 1]);
    arg1.push([0, 1]);
    tester1(arg1);

    let mut arg2 = Vec::new();
    arg2.push([0, 1]);
    arg2.push([0, 2]);
    tester2(arg2);

    arg1.push([0, 1]);
    tester2(arg1);

    1
}
