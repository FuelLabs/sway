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
fn expected_behavior_inlined() -> u64 {
    let a = A { a: 11 };
    let mut b = B { a: a, x: (11, 11) };

    let mut ptr_b = ptr_inl(b);
    // ERROR: We expect these pointers NOT to be equal but they are.
    //        Means the copy semantic is broken in this case. 
    assert(__eq(__addr_of(b), ptr_b));

    42
}

#[test]
fn expected_behavior_not_inlined() -> u64 {
    let a = A { a: 11 };
    let mut b = B { a: a, x: (11, 11) };

    let mut ptr_b = ptr(b);
    // ERROR: We expect these pointers NOT to be equal but they are.
    //        Means the copy semantic is broken in this case. 
    assert(__eq(__addr_of(b), ptr_b));

    42
}

// This version of `ptr` will get inlined.
fn ptr_inl<T>(t: T) -> raw_ptr {
    __addr_of(t)
}

#[inline(never)]
fn ptr<T>(t: T) -> raw_ptr {
    __addr_of(t)
}

// IR output when using installed version of Forc 0.46.0.
// No copy of the struct is made when calling `ptr`.

// script {
//     entry fn main() -> u64, !1 {
//         entry():
//         v0 = const u64 0, !2
//         ret u64 v0
//     }

//     entry fn expected_behavior_inlined() -> u64, !5 {
//         local { u64 } __anon_0
//         local { u64, u64 } __anon_1
//         local mut { { u64 }, { u64, u64 } } __anon_2

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
//         v17 = get_local ptr { { u64 }, { u64, u64 } }, __anon_2
//         v18 = ptr_to_int v17 to u64, !15
//         v19 = get_local ptr { { u64 }, { u64, u64 } }, __anon_2, !16
//         v20 = ptr_to_int v19 to u64, !17
//         v21 = cmp eq v20 v18
//         v22 = const bool false, !19
//         v23 = cmp eq v21 v22, !23
//         cbr v23, assert_1_block0(), assert_1_block1(), !24

//         assert_1_block0():
//         v24 = const u64 18446744073709486084, !26
//         revert v24, !30

//         assert_1_block1():
//         v25 = const u64 42, !31
//         ret u64 v25
//     }

//     entry fn expected_behavior_not_inlined() -> u64, !34 {
//         local { u64 } __anon_0
//         local { u64, u64 } __anon_1
//         local mut { { u64 }, { u64, u64 } } __anon_2

//         entry():
//         v0 = get_local ptr { u64 }, __anon_0, !35
//         v1 = const u64 0
//         v2 = get_elem_ptr v0, ptr u64, v1
//         v3 = const u64 11, !36
//         store v3 to v2, !35
//         v4 = get_local ptr { u64 }, __anon_0, !37
//         v5 = get_local ptr { u64, u64 }, __anon_1, !38
//         v6 = const u64 0
//         v7 = get_elem_ptr v5, ptr u64, v6, !38
//         v8 = const u64 11, !39
//         store v8 to v7, !38
//         v9 = const u64 1
//         v10 = get_elem_ptr v5, ptr u64, v9, !38
//         v11 = const u64 11, !40
//         store v11 to v10, !38
//         v12 = get_local ptr { { u64 }, { u64, u64 } }, __anon_2, !41
//         v13 = const u64 0
//         v14 = get_elem_ptr v12, ptr { u64 }, v13
//         mem_copy_val v14, v4
//         v15 = const u64 1
//         v16 = get_elem_ptr v12, ptr { u64, u64 }, v15
//         mem_copy_val v16, v5
//         v17 = get_local ptr { { u64 }, { u64, u64 } }, __anon_2
//         v18 = call ptr_4(v17)
//         v19 = get_local ptr { { u64 }, { u64, u64 } }, __anon_2, !42
//         v20 = ptr_to_int v19 to u64, !43
//         v21 = cmp eq v20 v18
//         v22 = const bool false, !19
//         v23 = cmp eq v21 v22, !45
//         cbr v23, assert_5_block0(), assert_5_block1(), !46

//         assert_5_block0():
//         v24 = const u64 18446744073709486084, !26
//         revert v24, !47

//         assert_5_block1():
//         v25 = const u64 42, !48
//         ret u64 v25
//     }

//     fn ptr_4(t: ptr { { u64 }, { u64, u64 } }) -> u64, !51 {
//         entry(t: ptr { { u64 }, { u64, u64 } }):
//         v0 = ptr_to_int t to u64, !52
//         ret u64 v0
//     }
// }

// IR output when using the latest master branch of Sway.
// Simpler code then in 0.46.0, but again no copy of the struct is made when calling `ptr`.

// script {
//     entry fn main() -> u64, !1 {
//         entry():
//         v0 = const u64 0, !2
//         ret u64 v0
//     }

//     entry fn expected_behavior_inlined() -> u64, !5 {
//         local mut { { u64 }, { u64, u64 } } __anon_2

//         entry():
//         v0 = get_local ptr { { u64 }, { u64, u64 } }, __anon_2, !6
//         v1 = const u64 0
//         v2 = get_elem_ptr v0, ptr { u64 }, v1
//         v3 = const u64 0
//         v4 = get_elem_ptr v2, ptr u64, v3
//         v5 = const u64 11, !7
//         store v5 to v4
//         v6 = const u64 1
//         v7 = get_elem_ptr v0, ptr { u64, u64 }, v6
//         v8 = const u64 0
//         v9 = get_elem_ptr v7, ptr u64, v8
//         v10 = const u64 11, !8
//         store v10 to v9
//         v11 = const u64 1
//         v12 = get_elem_ptr v7, ptr u64, v11
//         v13 = const u64 11, !9
//         store v13 to v12
//         v14 = get_local ptr { { u64 }, { u64, u64 } }, __anon_2
//         v15 = ptr_to_int v14 to u64, !12
//         v16 = get_local ptr { { u64 }, { u64, u64 } }, __anon_2, !13
//         v17 = ptr_to_int v16 to u64, !14
//         v18 = cmp eq v17 v15
//         v19 = const bool false, !16
//         v20 = cmp eq v18 v19, !20
//         cbr v20, assert_1_block0(), assert_1_block1(), !21

//         assert_1_block0():
//         v21 = const u64 18446744073709486084, !23
//         revert v21, !27

//         assert_1_block1():
//         v22 = const u64 42, !28
//         ret u64 v22
//     }

//     fn ptr_inl_0(t: ptr { { u64 }, { u64, u64 } }) -> u64, !29 {
//         entry(t: ptr { { u64 }, { u64, u64 } }):
//         v0 = ptr_to_int t to u64, !11
//         ret u64 v0
//     }

//     entry fn expected_behavior_not_inlined() -> u64, !32 {
//         local mut { { u64 }, { u64, u64 } } __anon_2

//         entry():
//         v0 = get_local ptr { { u64 }, { u64, u64 } }, __anon_2, !33
//         v1 = const u64 0
//         v2 = get_elem_ptr v0, ptr { u64 }, v1
//         v3 = const u64 0
//         v4 = get_elem_ptr v2, ptr u64, v3
//         v5 = const u64 11, !34
//         store v5 to v4
//         v6 = const u64 1
//         v7 = get_elem_ptr v0, ptr { u64, u64 }, v6
//         v8 = const u64 0
//         v9 = get_elem_ptr v7, ptr u64, v8
//         v10 = const u64 11, !35
//         store v10 to v9
//         v11 = const u64 1
//         v12 = get_elem_ptr v7, ptr u64, v11
//         v13 = const u64 11, !36
//         store v13 to v12
//         v14 = get_local ptr { { u64 }, { u64, u64 } }, __anon_2
//         v15 = call ptr_inl_0(v14)
//         v16 = get_local ptr { { u64 }, { u64, u64 } }, __anon_2, !37
//         v17 = ptr_to_int v16 to u64, !38
//         v18 = cmp eq v17 v15
//         v19 = const bool false, !16
//         v20 = cmp eq v18 v19, !40
//         cbr v20, assert_5_block0(), assert_5_block1(), !41

//         assert_5_block0():
//         v21 = const u64 18446744073709486084, !23
//         revert v21, !42

//         assert_5_block1():
//         v22 = const u64 42, !43
//         ret u64 v22
//     }
// }
