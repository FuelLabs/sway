library;

fn literal_index() -> u64 {
    let ary = [1, 2, 3];
    ary[4]
}

// TODO: For reasoning behind not emitting error and possible alternatives see:
//         https://github.com/FuelLabs/sway/issues/7605
fn literal_index_const_generic<const N: u64>() -> u64 {
    let ary = [1; N];
    ary[4] // THIS SHOULD NOT EMIT OUT-OF-BOUNDS-ERROR.
}

const GLOBAL_I: u64 = 4;

fn global_const_index() -> u64 {
    let ary = [1, 2, 3];
    ary[GLOBAL_I]
}

// TODO: For reasoning behind not emitting error and possible alternatives see:
//         https://github.com/FuelLabs/sway/issues/7605
fn global_const_index_const_generic<const N: u64>() -> u64 {
    let ary = [1; N];
    ary[GLOBAL_I] // THIS SHOULD NOT EMIT OUT-OF-BOUNDS-ERROR.
}

fn local_const_index() -> u64 {
    const LOCAL_I: u64 = 4;
    let ary = [1, 2, 3];
    ary[LOCAL_I]
}

// TODO: For reasoning behind not emitting error and possible alternatives see:
//         https://github.com/FuelLabs/sway/issues/7605
fn local_const_index_const_generic<const N: u64>() -> u64 {
    const LOCAL_I: u64 = 4;
    let ary = [1; N];
    ary[LOCAL_I] // THIS SHOULD NOT EMIT OUT-OF-BOUNDS-ERROR.
}

#[test]
fn test() {
    let _ = literal_index();
    let _ = literal_index_const_generic::<0>();
    let _ = global_const_index();
    let _ = global_const_index_const_generic::<0>();
    let _ = local_const_index();
    let _ = local_const_index_const_generic::<0>();
}
