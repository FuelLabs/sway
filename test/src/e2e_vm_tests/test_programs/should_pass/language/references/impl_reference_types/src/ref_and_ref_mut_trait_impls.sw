library;

trait Trait {
    fn val(self) -> u64;
}

impl Trait for &u64 {
    fn val(self) -> u64 {
        1
    }
}

impl Trait for &mut u64 {
    fn val(self) -> u64 {
        2
    }
}

impl Trait for & &u64 {
    fn val(self) -> u64 {
        1_1
    }
}

impl Trait for &mut &mut u64 {
    fn val(self) -> u64 {
        2_2
    }
}

impl Trait for & &mut u64 {
    fn val(self) -> u64 {
        1_2
    }
}

impl Trait for &mut & u64 {
    fn val(self) -> u64 {
        2_1
    }
}

impl Trait for & & &u64 {
    fn val(self) -> u64 {
        1_1_1
    }
}

impl Trait for &mut &mut &mut u64 {
    fn val(self) -> u64 {
        2_2_2
    }
}

impl Trait for & & &mut u64 {
    fn val(self) -> u64 {
        1_1_2
    }
}

impl Trait for & &mut &mut u64 {
    fn val(self) -> u64 {
        1_2_2
    }
}

impl Trait for &mut & &mut u64 {
    fn val(self) -> u64 {
        2_1_2
    }
}

impl Trait for &mut &mut & u64 {
    fn val(self) -> u64 {
        2_2_1
    }
}

pub fn test() -> u64 {
    let mut x = 123u64;

    let r = &x;
    assert_eq(r.val(), 1);

    let r = &mut x;
    assert_eq(r.val(), 2);

    let r = & &x;
    assert_eq(r.val(), 1_1);

    let r = &mut &mut x;
    assert_eq(r.val(), 2_2);

    let r = & &mut x;
    assert_eq(r.val(), 1_2);

    let r = &mut & x;
    assert_eq(r.val(), 2_1);

    let r = & & & x;
    assert_eq(r.val(), 1_1_1);

    let r = &mut &mut &mut x;
    assert_eq(r.val(), 2_2_2);

    let r = & & &mut x;
    assert_eq(r.val(), 1_1_2);

    let r = & &mut &mut x;
    assert_eq(r.val(), 1_2_2);

    let r = &mut & &mut x;
    assert_eq(r.val(), 2_1_2);

    let r = &mut &mut & x;
    assert_eq(r.val(), 2_2_1);

    42
}
