// ignore garbage_collection_all_language_tests - needs a experimental feature
script;

struct S<const N: u64> {
    arr: [u64; N]
}

fn main(a: [u64; 2]) {
    // let _ = __dbg(S::<3>{ arr: [1u64, 2u64, 3u64] });
}

#[test]
fn run_main() {
    main([1, 2]);
}
