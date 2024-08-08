script;

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
    assert(*__elem_at(&a, 0), 1);
    a[0] = 2;
    assert(*__elem_at(&a, 0), 2);
    //a[5] = 2;
}
