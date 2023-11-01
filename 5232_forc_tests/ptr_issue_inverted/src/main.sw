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
fn ptr_issue_inverted_inlined() -> u64 {
    let a = A { a: 11 };
    let mut b = B { a: a, x: (11, 11) };

    let mut ptr_b = ptr_inl(b);

    ptr_b.write(B { a: A { a: 22 }, x: (22, 22)});

    assert(b.a.a == 22);
    assert(b.x.0 == 22);
    assert(b.x.1 == 22);

    42
}

#[test]
fn ptr_issue_inverted_not_inlined() -> u64 {
    let a = A { a: 11 };
    let mut b = B { a: a, x: (11, 11) };

    let mut ptr_b = ptr(b);

    ptr_b.write(B { a: A { a: 22 }, x: (22, 22)});

    assert(b.a.a == 22);
    assert(b.x.0 == 22);
    assert(b.x.1 == 22);

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

// Unexpectedly, the test passes on both v0.46.0 and the latest master.
// I expected it to fail on the v0.46.0 because I expected a copy of B
// to be created when calling `ptr` but that was not the case.

// Again, the generated code differs between the Sway versions, but in
// both versions, there is no copy created.

// v0.46.0
// script {
//     entry fn main() -> u64, !1 {
//         entry():
//         v0 = const u64 0, !2
//         ret u64 v0
//     }

//     entry fn ptr_issue_inverted_inlined() -> u64, !5 {
//         local { u64 } __anon_0
//         local { u64, u64 } __anon_1
//         local mut { { u64 }, { u64, u64 } } __anon_2
//         local { u64 } __anon_3
//         local { u64, u64 } __anon_4
//         local { { u64 }, { u64, u64 } } __anon_5
//         local { { u64 }, { u64, u64 } } __asm_arg

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
//         v19 = get_local ptr { u64 }, __anon_3, !16
//         v20 = const u64 0
//         v21 = get_elem_ptr v19, ptr u64, v20
//         v22 = const u64 22, !17
//         store v22 to v21, !16
//         v23 = get_local ptr { u64, u64 }, __anon_4, !18
//         v24 = const u64 0
//         v25 = get_elem_ptr v23, ptr u64, v24, !18
//         v26 = const u64 22, !19
//         store v26 to v25, !18
//         v27 = const u64 1
//         v28 = get_elem_ptr v23, ptr u64, v27, !18
//         v29 = const u64 22, !20
//         store v29 to v28, !18
//         v30 = get_local ptr { { u64 }, { u64, u64 } }, __anon_5, !21
//         v31 = const u64 0
//         v32 = get_elem_ptr v30, ptr { u64 }, v31
//         mem_copy_val v32, v19
//         v33 = const u64 1
//         v34 = get_elem_ptr v30, ptr { u64, u64 }, v33
//         mem_copy_val v34, v23
//         v35 = get_local ptr { { u64 }, { u64, u64 } }, __asm_arg
//         mem_copy_val v35, v30
//         v36 = const u64 24
//         v37 = asm(dst: v18, src: v35, count: v36) {
//             mcp    dst src count, !23
//         }
//         v38 = get_local ptr { { u64 }, { u64, u64 } }, __anon_2, !24
//         v39 = const u64 0
//         v40 = get_elem_ptr v38, ptr { u64 }, v39, !25
//         v41 = const u64 0
//         v42 = get_elem_ptr v40, ptr u64, v41, !26
//         v43 = load v42
//         v44 = const u64 22, !27
//         v45 = cmp eq v43 v44, !28
//         v46 = call assert_2(v45), !29
//         v47 = get_local ptr { { u64 }, { u64, u64 } }, __anon_2, !30
//         v48 = const u64 1
//         v49 = get_elem_ptr v47, ptr { u64, u64 }, v48, !31
//         v50 = const u64 0
//         v51 = get_elem_ptr v49, ptr u64, v50, !32
//         v52 = load v51
//         v53 = const u64 22, !33
//         v54 = cmp eq v52 v53, !34
//         v55 = call assert_2(v54), !35
//         v56 = get_local ptr { { u64 }, { u64, u64 } }, __anon_2, !36
//         v57 = const u64 1
//         v58 = get_elem_ptr v56, ptr { u64, u64 }, v57, !31
//         v59 = const u64 1
//         v60 = get_elem_ptr v58, ptr u64, v59, !37
//         v61 = load v60
//         v62 = const u64 22, !38
//         v63 = cmp eq v61 v62, !39
//         v64 = call assert_2(v63), !40
//         v65 = const u64 42, !41
//         ret u64 v65
//     }

//     pub fn assert_2(condition !43: bool) -> (), !44 {
//         entry(condition: bool):
//         v0 = const bool false, !46
//         v1 = cmp eq condition v0, !47
//         cbr v1, block0(), block1(), !47

//         block0():
//         v2 = const u64 18446744073709486084, !49
//         revert v2, !53

//         block1():
//         v3 = const unit ()
//         ret () v3
//     }

//     entry fn ptr_issue_inverted_not_inlined() -> u64, !56 {
//         local { u64 } __anon_0
//         local { u64, u64 } __anon_1
//         local mut { { u64 }, { u64, u64 } } __anon_2
//         local { u64 } __anon_3
//         local { u64, u64 } __anon_4
//         local { { u64 }, { u64, u64 } } __anon_5
//         local { { u64 }, { u64, u64 } } __asm_arg

//         entry():
//         v0 = get_local ptr { u64 }, __anon_0, !57
//         v1 = const u64 0
//         v2 = get_elem_ptr v0, ptr u64, v1
//         v3 = const u64 11, !58
//         store v3 to v2, !57
//         v4 = get_local ptr { u64 }, __anon_0, !59
//         v5 = get_local ptr { u64, u64 }, __anon_1, !60
//         v6 = const u64 0
//         v7 = get_elem_ptr v5, ptr u64, v6, !60
//         v8 = const u64 11, !61
//         store v8 to v7, !60
//         v9 = const u64 1
//         v10 = get_elem_ptr v5, ptr u64, v9, !60
//         v11 = const u64 11, !62
//         store v11 to v10, !60
//         v12 = get_local ptr { { u64 }, { u64, u64 } }, __anon_2, !63
//         v13 = const u64 0
//         v14 = get_elem_ptr v12, ptr { u64 }, v13
//         mem_copy_val v14, v4
//         v15 = const u64 1
//         v16 = get_elem_ptr v12, ptr { u64, u64 }, v15
//         mem_copy_val v16, v5
//         v17 = get_local ptr { { u64 }, { u64, u64 } }, __anon_2
//         v18 = call ptr_6(v17)
//         v19 = get_local ptr { u64 }, __anon_3, !64
//         v20 = const u64 0
//         v21 = get_elem_ptr v19, ptr u64, v20
//         v22 = const u64 22, !65
//         store v22 to v21, !64
//         v23 = get_local ptr { u64, u64 }, __anon_4, !66
//         v24 = const u64 0
//         v25 = get_elem_ptr v23, ptr u64, v24, !66
//         v26 = const u64 22, !67
//         store v26 to v25, !66
//         v27 = const u64 1
//         v28 = get_elem_ptr v23, ptr u64, v27, !66
//         v29 = const u64 22, !68
//         store v29 to v28, !66
//         v30 = get_local ptr { { u64 }, { u64, u64 } }, __anon_5, !69
//         v31 = const u64 0
//         v32 = get_elem_ptr v30, ptr { u64 }, v31
//         mem_copy_val v32, v19
//         v33 = const u64 1
//         v34 = get_elem_ptr v30, ptr { u64, u64 }, v33
//         mem_copy_val v34, v23
//         v35 = get_local ptr { { u64 }, { u64, u64 } }, __asm_arg
//         mem_copy_val v35, v30
//         v36 = const u64 24
//         v37 = asm(dst: v18, src: v35, count: v36) {
//             mcp    dst src count, !23
//         }
//         v38 = get_local ptr { { u64 }, { u64, u64 } }, __anon_2, !70
//         v39 = const u64 0
//         v40 = get_elem_ptr v38, ptr { u64 }, v39, !25
//         v41 = const u64 0
//         v42 = get_elem_ptr v40, ptr u64, v41, !26
//         v43 = load v42
//         v44 = const u64 22, !71
//         v45 = cmp eq v43 v44, !72
//         v46 = call assert_8(v45), !73
//         v47 = get_local ptr { { u64 }, { u64, u64 } }, __anon_2, !74
//         v48 = const u64 1
//         v49 = get_elem_ptr v47, ptr { u64, u64 }, v48, !31
//         v50 = const u64 0
//         v51 = get_elem_ptr v49, ptr u64, v50, !75
//         v52 = load v51
//         v53 = const u64 22, !76
//         v54 = cmp eq v52 v53, !77
//         v55 = call assert_8(v54), !78
//         v56 = get_local ptr { { u64 }, { u64, u64 } }, __anon_2, !79
//         v57 = const u64 1
//         v58 = get_elem_ptr v56, ptr { u64, u64 }, v57, !31
//         v59 = const u64 1
//         v60 = get_elem_ptr v58, ptr u64, v59, !80
//         v61 = load v60
//         v62 = const u64 22, !81
//         v63 = cmp eq v61 v62, !82
//         v64 = call assert_8(v63), !83
//         v65 = const u64 42, !84
//         ret u64 v65
//     }

//     fn ptr_6(t: ptr { { u64 }, { u64, u64 } }) -> u64, !87 {
//         entry(t: ptr { { u64 }, { u64, u64 } }):
//         v0 = ptr_to_int t to u64, !88
//         ret u64 v0
//     }

//     pub fn assert_8(condition !43: bool) -> (), !44 {
//         entry(condition: bool):
//         v0 = const bool false, !46
//         v1 = cmp eq condition v0, !47
//         cbr v1, block0(), block1(), !47

//         block0():
//         v2 = const u64 18446744073709486084, !49
//         revert v2, !89

//         block1():
//         v3 = const unit ()
//         ret () v3
//     }
// }

// Latest master
// script {
//     entry fn main() -> u64, !1 {
//         entry():
//         v0 = const u64 0, !2
//         ret u64 v0
//     }

//     entry fn ptr_issue_inverted_inlined() -> u64, !5 {
//         local mut { { u64 }, { u64, u64 } } __anon_2
//         local { { u64 }, { u64, u64 } } __asm_arg

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
//         v16 = get_local ptr { { u64 }, { u64, u64 } }, __asm_arg
//         v17 = const u64 0
//         v18 = const u64 0
//         v19 = get_elem_ptr v16, ptr u64, v17, v18
//         v20 = const u64 22, !13
//         store v20 to v19
//         v21 = const u64 1
//         v22 = const u64 0
//         v23 = get_elem_ptr v16, ptr u64, v21, v22
//         v24 = const u64 22, !14
//         store v24 to v23
//         v25 = const u64 1
//         v26 = const u64 1
//         v27 = get_elem_ptr v16, ptr u64, v25, v26
//         v28 = const u64 22, !15
//         store v28 to v27
//         v29 = const u64 24
//         v30 = asm(dst: v15, src: v16, count: v29) {
//             mcp    dst src count, !17
//         }
//         v31 = get_local ptr { { u64 }, { u64, u64 } }, __anon_2, !18
//         v32 = const u64 0
//         v33 = get_elem_ptr v31, ptr { u64 }, v32, !19
//         v34 = const u64 0
//         v35 = get_elem_ptr v33, ptr u64, v34, !20
//         v36 = load v35
//         v37 = const u64 22, !21
//         v38 = cmp eq v36 v37, !22
//         v39 = call assert_8(v38), !23
//         v40 = get_local ptr { { u64 }, { u64, u64 } }, __anon_2, !24
//         v41 = const u64 1
//         v42 = get_elem_ptr v40, ptr { u64, u64 }, v41, !25
//         v43 = const u64 0
//         v44 = get_elem_ptr v42, ptr u64, v43, !26
//         v45 = load v44
//         v46 = const u64 22, !27
//         v47 = cmp eq v45 v46, !28
//         v48 = call assert_8(v47), !29
//         v49 = get_local ptr { { u64 }, { u64, u64 } }, __anon_2, !30
//         v50 = const u64 1
//         v51 = get_elem_ptr v49, ptr { u64, u64 }, v50, !25
//         v52 = const u64 1
//         v53 = get_elem_ptr v51, ptr u64, v52, !31
//         v54 = load v53
//         v55 = const u64 22, !32
//         v56 = cmp eq v54 v55, !33
//         v57 = call assert_8(v56), !34
//         v58 = const u64 42, !35
//         ret u64 v58
//     }

//     fn ptr_inl_0(t: ptr { { u64 }, { u64, u64 } }) -> u64, !36 {
//         entry(t: ptr { { u64 }, { u64, u64 } }):
//         v0 = ptr_to_int t to u64, !11
//         ret u64 v0
//     }

//     entry fn ptr_issue_inverted_not_inlined() -> u64, !39 {
//         local mut { { u64 }, { u64, u64 } } __anon_2
//         local { { u64 }, { u64, u64 } } __asm_arg

//         entry():
//         v0 = get_local ptr { { u64 }, { u64, u64 } }, __anon_2, !40
//         v1 = const u64 0
//         v2 = get_elem_ptr v0, ptr { u64 }, v1
//         v3 = const u64 0
//         v4 = get_elem_ptr v2, ptr u64, v3
//         v5 = const u64 11, !41
//         store v5 to v4
//         v6 = const u64 1
//         v7 = get_elem_ptr v0, ptr { u64, u64 }, v6
//         v8 = const u64 0
//         v9 = get_elem_ptr v7, ptr u64, v8
//         v10 = const u64 11, !42
//         store v10 to v9
//         v11 = const u64 1
//         v12 = get_elem_ptr v7, ptr u64, v11
//         v13 = const u64 11, !43
//         store v13 to v12
//         v14 = get_local ptr { { u64 }, { u64, u64 } }, __anon_2
//         v15 = call ptr_inl_0(v14)
//         v16 = get_local ptr { { u64 }, { u64, u64 } }, __asm_arg
//         v17 = const u64 0
//         v18 = const u64 0
//         v19 = get_elem_ptr v16, ptr u64, v17, v18
//         v20 = const u64 22, !44
//         store v20 to v19
//         v21 = const u64 1
//         v22 = const u64 0
//         v23 = get_elem_ptr v16, ptr u64, v21, v22
//         v24 = const u64 22, !45
//         store v24 to v23
//         v25 = const u64 1
//         v26 = const u64 1
//         v27 = get_elem_ptr v16, ptr u64, v25, v26
//         v28 = const u64 22, !46
//         store v28 to v27
//         v29 = const u64 24
//         v30 = asm(dst: v15, src: v16, count: v29) {
//             mcp    dst src count, !17
//         }
//         v31 = get_local ptr { { u64 }, { u64, u64 } }, __anon_2, !47
//         v32 = const u64 0
//         v33 = get_elem_ptr v31, ptr { u64 }, v32, !19
//         v34 = const u64 0
//         v35 = get_elem_ptr v33, ptr u64, v34, !20
//         v36 = load v35
//         v37 = const u64 22, !48
//         v38 = cmp eq v36 v37, !49
//         v39 = call assert_8(v38), !50
//         v40 = get_local ptr { { u64 }, { u64, u64 } }, __anon_2, !51
//         v41 = const u64 1
//         v42 = get_elem_ptr v40, ptr { u64, u64 }, v41, !25
//         v43 = const u64 0
//         v44 = get_elem_ptr v42, ptr u64, v43, !52
//         v45 = load v44
//         v46 = const u64 22, !53
//         v47 = cmp eq v45 v46, !54
//         v48 = call assert_8(v47), !55
//         v49 = get_local ptr { { u64 }, { u64, u64 } }, __anon_2, !56
//         v50 = const u64 1
//         v51 = get_elem_ptr v49, ptr { u64, u64 }, v50, !25
//         v52 = const u64 1
//         v53 = get_elem_ptr v51, ptr u64, v52, !57
//         v54 = load v53
//         v55 = const u64 22, !58
//         v56 = cmp eq v54 v55, !59
//         v57 = call assert_8(v56), !60
//         v58 = const u64 42, !61
//         ret u64 v58
//     }

//     pub fn assert_8(condition !63: bool) -> (), !64 {
//         entry(condition: bool):
//         v0 = const bool false, !66
//         v1 = cmp eq condition v0, !67
//         cbr v1, block0(), block1(), !67

//         block0():
//         v2 = const u64 18446744073709486084, !69
//         revert v2, !73

//         block1():
//         v3 = const unit ()
//         ret () v3
//     }
// }
