library;

use std::u128::*;
use ::utils::*;

pub fn result_impl_test() {
    let res = U128::from((0, 13)).as_u64();
    assert(!Result::dummy(false).unwrap());
    assert(res.unwrap_or(5) == 13);
}

pub fn generic_impl_self_test() {
    let a = double_identity(true, true);
    assert(a.first);
    assert(a.second);

    let b = double_identity(10u32, 43u64);
    assert(b.first == 10u32);
    assert(b.second == 43u64);

    let c = double_identity2(10u8, 1u8);
    assert(c.first == 10u8);
    assert(c.second == 1u8);

    let d = DoubleIdentity {
        first: 1u8,
        second: 2u8,
        third: 40u64,
    };
    assert(d.third == 40u64);

    let e = d.get_second();
    assert(e == 2u8);

    let f: DoubleIdentity<bool, bool> = double_identity(true, true);
    assert(f.first && f.second);

    let g: DoubleIdentity<u32, u64> = double_identity(10u32, 43u64);
    assert((g.first + 33u32) == g.second);

    let h = DoubleIdentity::<u64, bool>::new(3u64, false);
    assert(!h.second);

    let i = crazy(7u8, 10u8);
    assert(i == 10u8);

    let k = d.add();
    assert(k == 3u8);

    let l = Data::<bool>::new(false);
    assert(!l.value);

    let m: DoubleIdentity<Data<u8>, Data<u64>> = DoubleIdentity {
        first: Data { value: 1u8 },
        second: Data { value: 2u8 },
        third: 1u64,
    };
    assert(m.second.value == (m.first.value + m.third));

    let n = DoubleIdentity::<Data<u8>, Data<u8>>::new(Data::<u8>::new(3u8), Data::<u8>::new(4u8));
    assert(n.third == 10u64);

    let o: DoubleIdentity<bool, bool> = double_identity(true, true);
    assert(o.first && o.second);

    let p = MyOption::Some::<bool>(false);
    assert(p.is_some());

    let q = MyOption::Some::<()>(());
    assert(q.is_some());

    let r = MyOption::<u32>::some(5u32);
    assert(r.is_some());

    let s = MyOption::Some(0u8);
    assert(s.is_some());

    let t = MyOption::<u64>::none();
    assert(t.is_none());

    let u = DoubleIdentity::<Data<u8>, Data<u8>>::new(Data::<u8>::new(3u8), Data::<u8>::new(4u8));
    assert(u.first.value + u.second.value == 7u8);
}
