script;

fn revert_01() -> u32 {
    __revert(0);

    422
}

fn revert_02() -> u32 {
    __revert(0);
 
    return 2;
}

fn revert_03(a: u64) -> u32 {
    if a > 0 {
        __revert(0);
        return 1;
    }
    else {
        return 0;
    }
 
    return 3;
}

fn std_revert_04() -> u32 {
    revert(0);

    return 4;
}


fn main() -> u32 {
    let _ = revert_01();
    let _ = revert_02();
    let _ = revert_03(0);
    let _ = std_revert_04();
    0
}



