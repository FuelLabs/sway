script;

struct Foo {}

impl Foo {
    pub fn rec_0(self) -> u32 { self.rec_0() }

    pub fn rec_1(self) -> u32 { self.rec_2() }
    pub fn rec_2(self) -> u32 { self.rec_1() }

    pub fn rec_3(self) -> u32 { self.rec_4() }
    pub fn rec_4(self) -> u32 { self.rec_5() }
    pub fn rec_5(self) -> u32 { self.rec_3() }
}

fn main() -> u32 {
    0
}
