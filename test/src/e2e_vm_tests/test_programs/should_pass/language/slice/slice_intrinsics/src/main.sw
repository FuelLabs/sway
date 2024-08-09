script;

fn assert_refs<T>(l: &T, r: &T)
where
    T: Eq + AbiEncode
{
    if *l != *r {
        __log(*l);
        __log(*r);
        __revert(1)
    }
}

fn assert_mut_refs<T>(l: &mut T, r: &mut T)
where
    T: Eq + AbiEncode
{
    if *l != *r {
        __log(*l);
        __log(*r);
        __revert(1)
    }
}

fn assert<T>(l: T, r: T)
where
    T: Eq + AbiEncode
{
    if l != r {
        __log(l);
        __log(r);
        __revert(1)
    }
}

fn main()  {
    let mut a: [u64; 5] = [1, 2, 3, 4, 5];

    assert(a[0], 1);
    assert_refs(&a[0], &1);
    assert_mut_refs(&mut a[0], &mut 1);
    a[0] = 2;
    assert(a[0], 2);
    assert_refs(&a[0], &2);
    assert_mut_refs(&mut a[0], &mut 2);

    //a[5] = 2;
}
