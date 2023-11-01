script;

fn main() -> u64 {
   0
}

struct A {
  a: u64,
}

struct B {
  a: A,
  x: (u64, u64),
}

#[test]
fn ptr_issue() -> u64 {
    let a = A { a: 11 };
    let mut b = B { a: a, x: (11, 11) };

    let mut ptr_b = ptr(b);

    b.x.0 = 111;
    assert(b.x.0 == 111);
    // FAILS: b is not changed at all.
    assert(ptr_b.read::<B>().x.0 == 111);

    42
}

fn ptr<T>(t: T) -> raw_ptr {
    __addr_of(t)
}

// The original code sample can be found here: https://github.com/FuelLabs/sway/issues/5232

// When running the Forc test using the latest master the structs are not copied at all.
// In addition, `b` is not changed to 111 (no reason to because it's not used in code later on)
// and the assert becomes `111 == 111`.

// forc 0.46.1

// cargo run --bin forc -- test --ir -p ~/p/swaylab/I\ 5232\ Wrapping\ __addr_of\ in\ a\ function\ results\ in\ incorrect\ compilation/forc_tests/ptr_issue/

// script {
//     entry fn main() -> u64, !1 {
//         entry():
//         v0 = const u64 0, !2
//         ret u64 v0
//     }

//     entry fn ptr_issue() -> u64, !5 {
//         local { { u64 }, { u64, u64 } } __ptr_to_int_arg

//         entry():
//         v0 = get_local ptr { { u64 }, { u64, u64 } }, __ptr_to_int_arg
//         v1 = const u64 0
//         v2 = const u64 0
//         v3 = get_elem_ptr v0, ptr u64, v1, v2
//         v4 = const u64 11, !6
//         store v4 to v3
//         v5 = const u64 1
//         v6 = const u64 0
//         v7 = get_elem_ptr v0, ptr u64, v5, v6
//         v8 = const u64 11, !7
//         store v8 to v7
//         v9 = const u64 1
//         v10 = const u64 1
//         v11 = get_elem_ptr v0, ptr u64, v9, v10
//         v12 = const u64 11, !8
//         store v12 to v11
//         v13 = ptr_to_int v0 to u64, !11
//         v14 = const u64 111, !12
//         v15 = const u64 111, !13
//         v16 = cmp eq v14 v15, !14
//         v17 = call assert_1(v16), !15
//         v18 = asm(ptr: v13) -> ptr { { u64 }, { u64, u64 } } ptr {
//         }
//         v19 = const u64 1
//         v20 = const u64 0
//         v21 = get_elem_ptr v18, ptr u64, v19, v20
//         v22 = load v21
//         v23 = const u64 111, !16
//         v24 = cmp eq v22 v23, !17
//         v25 = call assert_1(v24), !18
//         v26 = const u64 42, !19
//         ret u64 v26
//     }

//     pub fn assert_1(condition !21: bool) -> (), !22 {
//         entry(condition: bool):
//         v0 = const bool false, !24
//         v1 = cmp eq condition v0, !25
//         cbr v1, block0(), block1(), !25

//         block0():
//         v2 = const u64 18446744073709486084, !27
//         revert v2, !31

//         block1():
//         v3 = const unit ()
//         ret () v3
//     }
// }

// When running with the installed version of Forc copies are created.

// forc 0.46.0

// forc test --ir -p ~/p/swaylab/I\ 5232\ Wrapping\ __addr_of\ in\ a\ function\ results\ in\ incorrect\ compilation/forc_tests/ptr_issue/

// script {
//     entry fn main() -> u64, !1 {
//         entry():
//         v0 = const u64 0, !2
//         ret u64 v0
//     }

//     entry fn ptr_issue() -> u64, !5 {
//         local { u64 } __anon_0
//         local { u64, u64 } __anon_1
//         local { { u64 }, { u64, u64 } } __anon_2
//         local { { u64 }, { u64, u64 } } __anon_3
//         local { { u64 }, { u64, u64 } } __ptr_to_int_arg
//         local mut { { u64 }, { u64, u64 } } b

//         entry():
//         v0 = get_local ptr { u64 }, __anon_0, !6
//         v1 = const u64 0
//         v2 = get_elem_ptr v0, ptr u64, v1
//         v3 = const u64 11, !7
//         store v3 to v2, !6
//         v4 = get_local ptr { u64 }, __anon_0, !8
//         v5 = get_local ptr { u64, u64 }, __anon_1, !9
//         v6 = const u64 0
//         v7 = get_elem_ptr v5, ptr u64, v6, !9
//         v8 = const u64 11, !10
//         store v8 to v7, !9
//         v9 = const u64 1
//         v10 = get_elem_ptr v5, ptr u64, v9, !9
//         v11 = const u64 11, !11
//         store v11 to v10, !9
//         v12 = get_local ptr { { u64 }, { u64, u64 } }, __anon_2, !12
//         v13 = const u64 0
//         v14 = get_elem_ptr v12, ptr { u64 }, v13
//         mem_copy_val v14, v4
//         v15 = const u64 1
//         v16 = get_elem_ptr v12, ptr { u64, u64 }, v15
//         mem_copy_val v16, v5
//         v17 = get_local ptr { { u64 }, { u64, u64 } }, b, !13
//         mem_copy_val v17, v12
//         v18 = get_local ptr { { u64 }, { u64, u64 } }, __ptr_to_int_arg
//         mem_copy_val v18, v12
//         v19 = ptr_to_int v18 to u64, !16
//         v20 = get_local ptr { { u64 }, { u64, u64 } }, b, !17
//         v21 = const u64 1
//         v22 = const u64 0
//         v23 = get_elem_ptr v20, ptr u64, v21, v22, !17
//         v24 = const u64 111, !18
//         store v24 to v23, !17
//         v25 = get_local ptr { { u64 }, { u64, u64 } }, b, !19
//         v26 = const u64 1
//         v27 = get_elem_ptr v25, ptr { u64, u64 }, v26, !20
//         v28 = const u64 0
//         v29 = get_elem_ptr v27, ptr u64, v28, !21
//         v30 = load v29
//         v31 = const u64 111, !22
//         v32 = cmp eq v30 v31, !23
//         v33 = call assert_1(v32), !24
//         v34 = asm(ptr: v19) -> ptr { { u64 }, { u64, u64 } } ptr {
//         }
//         v35 = get_local ptr { { u64 }, { u64, u64 } }, __anon_3
//         mem_copy_val v35, v34
//         v36 = const u64 1
//         v37 = get_elem_ptr v35, ptr { u64, u64 }, v36, !20
//         v38 = const u64 0
//         v39 = get_elem_ptr v37, ptr u64, v38, !25
//         v40 = load v39
//         v41 = const u64 111, !26
//         v42 = cmp eq v40 v41, !27
//         v43 = call assert_1(v42), !28
//         v44 = const u64 42, !29
//         ret u64 v44
//     }

//     pub fn assert_1(condition !31: bool) -> (), !32 {
//         entry(condition: bool):
//         v0 = const bool false, !34
//         v1 = cmp eq condition v0, !35
//         cbr v1, block0(), block1(), !35

//         block0():
//         v2 = const u64 18446744073709486084, !37
//         revert v2, !41

//         block1():
//         v3 = const unit ()
//         ret () v3
//     }
// }