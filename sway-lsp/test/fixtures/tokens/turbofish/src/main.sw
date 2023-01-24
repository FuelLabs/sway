contract;

struct A {
    b: u32
}

impl A {
    pub fn method<T>(self, a: T){}
}

#[storage(read)]
pub fn g(amount: u64, to: Identity) {
    let s = A{b: 0};
    fun::<Option<u32>>(Option::None);
    s.method::<Option<u32>>(Option::None);
    let a = Option::None::<Option<u32>>;
}

fn fun<T>(t: T){}
