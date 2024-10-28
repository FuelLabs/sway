library;

/// This is only useful for the e2e harness setup, because
/// no one else knows how to "decode" this into meaningful
// textual logs.
pub trait Dbg {
    fn dbg(self);
} {
    fn dbgln(self) {
        self.dbg();
        asm(ra: u64::max(), rb: 2) {
            logd ra rb zero zero;
        }
    }
}

impl Dbg for str {
    fn dbg(self) {
        let encoded = encode(self);
        asm(ra: u64::max(), ptr: encoded.ptr(), len: encoded.len::<u8>()) {
            logd ra zero ptr len;
        }
    }
}


impl Dbg for u64 {
    fn dbg(self) {
        let encoded = encode(self);
        asm(ra: u64::max(), ptr: encoded.ptr(), len: encoded.len::<u8>()) {
           logd ra one ptr len;
        }
    }
}

pub struct NewLine {}

pub fn new_line() -> NewLine {
    NewLine { }
}

impl Dbg for NewLine {
    fn dbg(self) {
        asm(ra: u64::max(), rb: 2) {
            logd ra rb zero zero;
        }
    }
}

impl<A, B> Dbg for (A, B)
where
    A: Dbg,
    B: Dbg,
{
    fn dbg(self) {
        self.0.dbg();
        self.1.dbg();
    }
}

impl<A, B, C> Dbg for (A, B, C)
where
    A: Dbg,
    B: Dbg,
    C: Dbg,
{
    #[allow(dead_code)]
    fn dbg(self) {
        self.0.dbg();
        self.1.dbg();
        self.2.dbg();
    }
}

impl<A, B, C, D> Dbg for (A, B, C, D)
where
    A: Dbg,
    B: Dbg,
    C: Dbg,
    D: Dbg
{
    fn dbg(self) {
        self.0.dbg();
        self.1.dbg();
        self.2.dbg();
        self.3.dbg();
    }
}


impl<A, B, C, D, E> Dbg for (A, B, C, D, E)
where
    A: Dbg,
    B: Dbg,
    C: Dbg,
    D: Dbg,
    E: Dbg,
{
    fn dbg(self) {
        self.0.dbg();
        self.1.dbg();
        self.2.dbg();
        self.3.dbg();
        self.4.dbg();
    }
}

impl<A, B, C, D, E, F> Dbg for (A, B, C, D, E, F)
where
    A: Dbg,
    B: Dbg,
    C: Dbg,
    D: Dbg,
    E: Dbg,
    F: Dbg,
{
    fn dbg(self) {
        self.0.dbg();
        self.1.dbg();
        self.2.dbg();
        self.3.dbg();
        self.4.dbg();
        self.5.dbg();
    }
}

impl<A, B, C, D, E, F, G> Dbg for (A, B, C, D, E, F, G)
where
    A: Dbg,
    B: Dbg,
    C: Dbg,
    D: Dbg,
    E: Dbg,
    F: Dbg,
    G: Dbg
{
    fn dbg(self) {
        self.0.dbg();
        self.1.dbg();
        self.2.dbg();
        self.3.dbg();
        self.4.dbg();
        self.5.dbg();
        self.6.dbg();
    }
}
