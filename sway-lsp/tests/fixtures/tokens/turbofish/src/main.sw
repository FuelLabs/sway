contract;

struct A {
    b: u32
}

impl A {
    pub fn method<T>(self, a: T){}
}

struct B<T> {}

#[storage(read)]
pub fn g(amount: u64, to: Identity) {
    let s = A{b: 0};
    fun::<Option<u32>>(Option::None);
    s.method::<Option<u32>>(Option::None);
    let a = Option::None::<Option<u32>>;
    let b = B::<Option<u32>>{};

    fun::<Option<Result<u32, u32>>>(Option::None);
    s.method::<Option<Result<u32, u32>>>(Option::None);
    let a = Option::None::<Option<Result<u32, u32>>>;
    let b = B::<Option<Result<u32, u32>>>{};
}

fn fun<T>(t: T){}
