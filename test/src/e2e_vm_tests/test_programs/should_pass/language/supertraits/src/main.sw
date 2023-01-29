script;

/// Inheritence tree:
///      A 
///      |
///      |
///      B     
///     /|
///    / |
///   C  |
///    \ |
///     \|
///      D
///

trait A {
    fn f(self) -> u64;
} {
    fn add_f(self, x: u64) -> u64 {
        self.f() + x
    }
    fn mul_f(self, x: u64) -> u64 {
        self.f() * x
    }
}

trait B: A {
    fn g(self) -> u64;
} {
    fn add_g(self, x: u64) -> u64 {
        self.g() + x
    }
    fn mul_g(self, x: u64) -> u64 {
        self.g() * x
    }

    // Test access to A's interface 
    fn b_calls_f(self) -> u64 {
        self.f() + 1
    }

    // Test access to A's methods 
    fn b_calls_add_f(self, x: u64) -> u64 {
        self.add_f(x) + 1
    }
    fn b_calls_mul_f(self, x: u64) -> u64 {
        self.mul_f(x) + 1
    }
}

trait C: B {
    fn h(self) -> u64;
} {
    fn add_h(self, x: u64) -> u64 {
        self.h() + x
    }
    fn mul_h(self, x: u64) -> u64 {
        self.h() * x
    }
    
    // Test access to A's interface 
    fn c_calls_f(self) -> u64 {
        self.f() + 2
    }

    // Test access to B's interface 
    fn c_calls_g(self) -> u64 {
        self.g() + 2
    }
    
    // Test access to A's methods 
    fn c_calls_add_f(self, x: u64) -> u64 {
        self.add_f(x) + 2
    }
    fn c_calls_mul_f(self, x: u64) -> u64 {
        self.mul_f(x) + 2
    }
    
    // Test access to B's methods 
    fn c_calls_add_g(self, x: u64) -> u64 {
        self.add_g(x) + 2
    }
    fn c_calls_mul_g(self, x: u64) -> u64 {
        self.mul_g(x) + 2
    }
    fn c_calls_b_calls_f(self) -> u64 {
        self.b_calls_f() + 2
    }
    fn c_calls_b_calls_add_f(self, x: u64) -> u64 {
        self.b_calls_add_f(x) + 2
    }
    fn c_calls_b_calls_mul_f(self, x: u64) -> u64 {
        self.b_calls_mul_f(x) + 2
    }
}

trait D: B + C {
    fn i(self) -> u64;
} {
    fn add_i(self, x: u64) -> u64 {
        self.i() + x
    }
    fn mul_i(self, x: u64) -> u64 {
        self.i() * x
    }

    // Test access to A's interface 
    fn d_calls_f(self) -> u64 {
        self.f() + 3
    }

    // Test access to B's interface 
    fn d_calls_g(self) -> u64 {
        self.g() + 3
    }

    // Test access to C's interface 
    fn d_calls_h(self) -> u64 {
        self.h() + 3
    }
    
    // Test access to A's methods 
    fn d_calls_add_f(self, x: u64) -> u64 {
        self.add_f(x) + 3
    }
    fn d_calls_mul_f(self, x: u64) -> u64 {
        self.mul_f(x) + 3
    }
    
    // Test access to B's methods 
    fn d_calls_add_g(self, x: u64) -> u64 {
        self.add_g(x) + 3
    }
    fn d_calls_mul_g(self, x: u64) -> u64 {
        self.mul_g(x) + 3
    }
    fn d_calls_b_calls_f(self) -> u64 {
        self.b_calls_f() + 3
    }
    fn d_calls_b_calls_add_f(self, x: u64) -> u64 {
        self.b_calls_add_f(x) + 3
    }
    fn d_calls_b_calls_mul_f(self, x: u64) -> u64 {
        self.b_calls_mul_f(x) + 3
    }

    // Test access to C's methods
    fn d_calls_add_h(self, x: u64) -> u64 {
        self.add_h(x) + 3
    }
    fn d_calls_mul_h(self, x: u64) -> u64 {
        self.mul_h(x) + 3
    }
    fn d_calls_c_calls_f(self) -> u64 {
        self.c_calls_f() + 3
    }
    fn d_calls_c_calls_g(self) -> u64 {
        self.c_calls_g() + 3
    }
    fn d_calls_c_calls_add_f(self, x: u64) -> u64 {
        self.c_calls_add_f(x) + 3
    }
    fn d_calls_c_calls_mul_f(self, x: u64) -> u64 {
        self.c_calls_mul_f(x) + 3
    }
    fn d_calls_c_calls_add_g(self, x: u64) -> u64 {
        self.c_calls_add_g(x) + 3
    }
    fn d_calls_c_calls_mul_g(self, x: u64) -> u64 {
        self.c_calls_mul_g(x) + 3
    }
    fn d_calls_c_calls_b_calls_f(self) -> u64 {
        self.c_calls_b_calls_f() + 3
    }
    fn d_calls_c_calls_b_calls_add_f(self, x: u64) -> u64 {
        self.c_calls_b_calls_add_f(x) + 3
    }
    fn d_calls_c_calls_b_calls_mul_f(self, x: u64) -> u64 {
        self.c_calls_b_calls_mul_f(x) + 3
    }
}

struct S {
    x: u64,
    y: u64,
    z: u64,
    w: u64,
}

impl A for S {
    fn f(self) -> u64 {
        self.x
    }
}

impl B for S {
    fn g(self) -> u64 {
        self.y
    }
}

impl C for S {
    fn h(self) -> u64 {
        self.z
    }
}

impl D for S {
    fn i(self) -> u64 {
        self.w
    }
}

struct U {
    x: u64,
    y: u64,
    z: u64,
    w: u64,
}

impl A for U {
    fn f(self) -> u64 {
        self.x + 1
    }
}

impl B for U {
    fn g(self) -> u64 {
        self.y + 1
    }
}

impl C for U {
    fn h(self) -> u64 {
        self.z + 1
    }
}

fn main() -> bool {
    let s = S {
        x: 42,
        y: 2,
        z: 3,
        w: 4,
    };

    // To follow all the tests below, note that 
    // * each call from B (i.e. call with `calls_b`) adds 1
    // * each call from C (i.e. call with `calls_b`) adds 2
    // * each call from D (i.e. call with `calls_b`) adds 3    

    // Test access to f()
    assert(s.f() == s.x);
    assert(s.b_calls_f() == s.x + 1);
    assert(s.c_calls_f() == s.x + 2);
    assert(s.d_calls_f() == s.x + 3);
  
    // Test access to add_f()
    assert(s.add_f(5) == s.x + 5);
    assert(s.b_calls_add_f(5) == s.x + 5 + 1 );
    assert(s.c_calls_add_f(5) == s.x + 5 + 2);
    assert(s.c_calls_b_calls_add_f(5) == s.x + 5 + 1 + 2);
    assert(s.d_calls_add_f(5) == s.x + 5 + 3);
    assert(s.d_calls_b_calls_add_f(5) == s.x + 5 + 1 + 3);
    assert(s.d_calls_c_calls_add_f(5) == s.x + 5 + 2 + 3);
    assert(s.d_calls_c_calls_b_calls_add_f(5) == s.x + 5 + 1 + 2 + 3);

    // Test access to mul_f()
    assert(s.mul_f(5) == s.x * 5);
    assert(s.b_calls_mul_f(5) == s.x * 5 + 1 );
    assert(s.c_calls_mul_f(5) == s.x * 5 + 2);
    assert(s.c_calls_b_calls_mul_f(5) == s.x * 5 + 1 + 2);
    assert(s.d_calls_mul_f(5) == s.x * 5 + 3);
    assert(s.d_calls_b_calls_mul_f(5) == s.x * 5 + 1 + 3);
    assert(s.d_calls_c_calls_mul_f(5) == s.x * 5 + 2 + 3);
    assert(s.d_calls_c_calls_b_calls_mul_f(5) == s.x * 5 + 1 + 2 + 3);

    // Test access to g()
    assert(s.g() == s.y);
    assert(s.c_calls_g() == s.y + 2);
    assert(s.d_calls_g() == s.y + 3);
  
    // Test access to add_g()
    assert(s.add_g(5) == s.y + 5);
    assert(s.c_calls_add_g(5) == s.y + 5 + 2);
    assert(s.d_calls_add_g(5) == s.y + 5 + 3);
    assert(s.d_calls_c_calls_add_g(5) == s.y + 5 + 2 + 3);

    // Test access to mul_g()
    assert(s.mul_g(5) == s.y * 5);
    assert(s.c_calls_mul_g(5) == s.y * 5 + 2);
    assert(s.d_calls_mul_g(5) == s.y * 5 + 3);
    assert(s.d_calls_c_calls_mul_g(5) == s.y * 5 + 2 + 3);

    // Test access to h()
    assert(s.h() == s.z);
    assert(s.d_calls_h() == s.z + 3);
  
    // Test access to add_h()
    assert(s.add_h(5) == s.z + 5);
    assert(s.d_calls_add_h(5) == s.z + 5 + 3);

    // Test access to mul_h()
    assert(s.mul_h(5) == s.z * 5);
    assert(s.d_calls_mul_h(5) == s.z * 5 + 3);

    // Test access to i()
    assert(s.i() == s.w);
  
    // Test access to add_i()
    assert(s.add_i(5) == s.w + 5);

    // Test access to mul_i()
    assert(s.mul_i(5) == s.w * 5);

    let u = U {
        x: 5,
        y: 6,
        z: 7,
        w: 8,
    };

    // To follow all the tests below, note that 
    // * each call from B (i.e. call with `calls_b`) adds 1
    // * each call from C (i.e. call with `calls_b`) adds 2
    // Note: no calls from D are allowed because U doesn't implement D

    // Test access to f()
    assert(u.f() == (u.x + 1));
    assert(u.b_calls_f() == (u.x + 1) + 1);
    assert(u.c_calls_f() == (u.x + 1) + 2);
  
    // Test access to add_f()
    assert(u.add_f(5) == (u.x + 1) + 5);
    assert(u.b_calls_add_f(5) == (u.x + 1) + 5 + 1 );
    assert(u.c_calls_add_f(5) == (u.x + 1) + 5 + 2);
    assert(u.c_calls_b_calls_add_f(5) == (u.x + 1) + 5 + 1 + 2);

    // Test access to mul_f()
    assert(u.mul_f(5) == (u.x + 1) * 5);
    assert(u.b_calls_mul_f(5) == (u.x + 1) * 5 + 1 );
    assert(u.c_calls_mul_f(5) == (u.x + 1) * 5 + 2);
    assert(u.c_calls_b_calls_mul_f(5) == (u.x + 1) * 5 + 1 + 2);

    // Test access to g()
    assert(u.g() == (u.y + 1));
    assert(u.c_calls_g() == (u.y + 1) + 2);
  
    // Test access to add_g()
    assert(u.add_g(5) == (u.y + 1) + 5);
    assert(u.c_calls_add_g(5) == (u.y + 1) + 5 + 2);

    // Test access to mul_g()
    assert(u.mul_g(5) == (u.y + 1) * 5);
    assert(u.c_calls_mul_g(5) == (u.y + 1) * 5 + 2);

    // Test access to h()
    assert(u.h() == (u.z + 1));
  
    // Test access to add_h()
    assert(u.add_h(5) == (u.z + 1) + 5);

    // Test access to mul_h()
    assert(u.mul_h(5) == (u.z + 1) * 5);
 
    true
}
