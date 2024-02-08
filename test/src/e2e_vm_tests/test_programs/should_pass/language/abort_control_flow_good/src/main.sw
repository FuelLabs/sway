script;

fn local_panic<T>() -> T {
    __revert(42)
}

fn main() -> u64 {
    // all of these should be okay, since
    // the branches that would have type errors abort control flow.
    let _x = if true { 42u64 } else { revert(0) };
    let _x: u64 = local_panic::<u64>();
    let _x = if let Result::Ok(ok) = Result::Ok::<u64, u64>(5) {
        ok
    } else {
        local_panic::<u64>()
    };
    let _x = if true {
        Result::Err::<u64, u32>(12)
    } else {
        return 10;
    };
    return 42;
}
