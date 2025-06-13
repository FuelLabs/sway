script;

struct A {
    a: u64,
}

impl A {
    fn f(ref mut self) {
        self.a = 0;
    }
}

fn main() {
    let mut a = A { a: 0 };
    a.f();
}

// check: local mut { u64 } a

// check: get_local ptr { u64 }, a

// check: $(a_var=$VAL) = get_local ptr { u64 }, a
// check: call $(f_method=$ID)($a_var)

// check: fn $f_method(self $MD: ptr { u64 }) -> ()
// check: entry(self: ptr { u64 }):

// check: $(idx_0=$VAL) = const u64 0
// check: $(a_ptr=$VAL) = get_elem_ptr $VAL, ptr u64, $idx_0
// check: $(zero_val=$VAL) = const u64 0
// check: store $zero_val to $a_ptr

// check: $(res=$VAL) = const unit ()
// check: ret () $res
