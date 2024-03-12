library;

impl &u64 {
    fn ref_u64(self) {}
}

impl &mut u64 {
    fn ref_mut_u64(self) {}
}

impl & &u64 {
    fn ref_ref_u64(self) {}
}

impl &mut &mut u64 {
    fn ref_mut_ref_mut_u64(self) {}
}

impl & &mut u64 {
    fn ref_ref_mut_u64(self) {}
}

impl &mut & u64 {
    fn ref_mut_ref_u64(self) {}
}

impl & & &u64 {
    fn ref_ref_ref_u64(self) {}
}

impl &mut &mut &mut u64 {
    fn ref_mut_ref_mut_ref_mut_u64(self) {}
}

impl & &mut &mut u64 {
    fn ref_ref_mut_ref_mut_u64(self) {}
}

impl &mut & &mut u64 {
    fn ref_mut_ref_ref_mut_u64(self) {}
}

impl &mut &mut & u64 {
    fn ref_mut_ref_mut_ref_u64(self) {}
}

pub fn test() {
    let mut x = 123u64;

    let r = &x;
    r.ref_mut_u64();

    let r = & &x;
    r.ref_mut_ref_mut_u64();
    r.ref_ref_mut_u64();
    r.ref_mut_ref_u64();

    let r = & & &x;
    r.ref_mut_ref_mut_ref_mut_u64();
    r.ref_ref_mut_ref_mut_u64();
    r.ref_mut_ref_ref_mut_u64();
    r.ref_mut_ref_mut_ref_u64();
}
