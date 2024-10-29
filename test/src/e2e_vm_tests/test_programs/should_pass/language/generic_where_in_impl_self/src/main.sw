script;

use std::assert::*;

trait Trait {
    #[inline(never)]
    fn method(self) -> u64;
}

impl Trait for u64 {
    #[inline(never)]
    fn method(self) -> u64{
        42
    }
}

struct CallTrait<V> {}

#[inline(never)]
fn call_trait<T>(t: T) -> u64 where T: Trait {
    t.method()
}

impl<K> CallTrait<K> where K: Trait {
    pub fn call_trait(self, key: K) -> u64 {
        call_trait(key)
    }
}

// https://github.com/FuelLabs/sway/issues/6382
trait Devour<T> { fn eat(t: T); }
struct Brain { }
struct Zombie { }

impl Devour<Brain> for Zombie {
    fn eat(_b: Brain) { }
}

fn feed<T, U>(t: T) where U: Devour<T> {
    U::eat(t);
}

fn main() -> bool {
    let _  = call_trait(1);
    let ct = CallTrait::<u64> {};
    assert(ct.call_trait(1) == 42);

    // https://github.com/FuelLabs/sway/issues/6382
    let b = Brain{};
    feed::<Brain, Zombie>(b);

    true
}
