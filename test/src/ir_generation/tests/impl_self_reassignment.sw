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

// check: local mut ptr { u64 } a

// check: get_ptr mut ptr { u64 } a, ptr { u64 }, 0

// check: $(a_ptr=$VAL) = get_ptr mut ptr { u64 } a, ptr { u64 }, 0
// check: call $(f_method=$ID)($a_ptr)

// check: fn $f_method(self $MD: { u64 }) -> ()
// nextln: entry:
// nextln: $(zero_val=$VAL) = const u64 0
// nextln: $VAL = insert_value self, { u64 }, $zero_val, 0
// nextln: $(res=$VAL) = const unit ()
// nextln: ret () $res
