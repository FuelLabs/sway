library;

impl &u64 {
    fn get_value(self) -> u64 {
        let ptr = asm(r: self) { r: raw_ptr };

        ptr.read::<u64>()
    }
    
    fn get_value_deref(self) -> u64 {
        *self
    }
}

impl &mut u64 {
    fn mut_get_value(self) -> u64 {
        let ptr = asm(r: self) { r: raw_ptr };

        ptr.read::<u64>()
    }
    
    fn mut_get_value_deref(self) -> u64 {
        *self
    }
}

impl & &u64 {
    fn get_value(self) -> u64 {
        let ptr = asm(r: self) { r: raw_ptr };

        ptr.read::<raw_ptr>().read::<u64>() * 2
    }
    
    fn get_value_deref(self) -> u64 {
        // Just to play a bit with the parser while we are here :-)
        assert_eq(**self * 2, 2 * **self);
        assert_eq(* * self * 2, 2 * * * self);
        assert_eq(**self*2, 2* **self);
        //                   ^ This space is needed to disambiguate from `pow`.

        **self * 2
    }
}

impl &mut &mut u64 {
    fn mut_get_value(self) -> u64 {
        let ptr = asm(r: self) { r: raw_ptr };

        ptr.read::<raw_ptr>().read::<u64>() * 2
    }
    
    fn mut_get_value_deref(self) -> u64 {
        // Just to play a bit with the parser while we are here :-)
        assert_eq(**self * 2, 2 * **self);
        assert_eq(* * self * 2, 2 * * * self);
        assert_eq(**self*2, 2* **self);
        //                   ^ This space is needed to disambiguate from `pow`.

        **self * 2
    }
}

impl &[u64;2] {
    fn get_value(self, index: u64) -> u64 {
        let ptr = asm(r: self) { r: raw_ptr };

        ptr.read::<[u64;2]>()[index]
    }

    fn get_value_deref(self, index: u64) -> u64 {
        (*self)[index]
    }

    fn get_value_deref_array(self, index: u64) -> u64 {
        self[index]
    }
}

impl &mut [u64;2] {
    fn mut_get_value(self, index: u64) -> u64 {
        let ptr = asm(r: self) { r: raw_ptr };

        ptr.read::<[u64;2]>()[index]
    }

    fn mut_get_value_deref(self, index: u64) -> u64 {
        (*self)[index]
    }

    fn mut_get_value_deref_array(self, index: u64) -> u64 {
        self[index]
    }
}

impl & &[u64;2] {
    fn get_value(self, index: u64) -> u64 {
        let ptr = asm(r: self) { r: raw_ptr };

        ptr.read::<raw_ptr>().read::<[u64;2]>()[index] * 2
    }

    fn get_value_deref(self, index: u64) -> u64 {
        (**self)[index] * 2
    }

    fn get_value_deref_array(self, index: u64) -> u64 {
        self[index] * 2
    }
}

impl &mut &mut[u64;2] {
    fn mut_get_value(self, index: u64) -> u64 {
        let ptr = asm(r: self) { r: raw_ptr };

        ptr.read::<raw_ptr>().read::<[u64;2]>()[index] * 2
    }

    fn mut_get_value_deref(self, index: u64) -> u64 {
        (**self)[index] * 2
    }

    fn mut_get_value_deref_array(self, index: u64) -> u64 {
        self[index] * 2
    }
}

trait Trait {
    fn trait_function() -> u64;
    fn trait_method(self) -> u64;
    fn trait_method_deref(self) -> u64;
}

impl Trait for &u64 {
    fn trait_function() -> u64 {
        64
    }

    fn trait_method(self) -> u64 {
        let ptr = asm(r: self) { r: raw_ptr };

        ptr.read::<u64>()
    }

    fn trait_method_deref(self) -> u64 {
        *self
    }
}

impl Trait for &mut u64 {
    fn trait_function() -> u64 {
        64
    }

    fn trait_method(self) -> u64 {
        let ptr = asm(r: self) { r: raw_ptr };

        ptr.read::<u64>()
    }

    fn trait_method_deref(self) -> u64 {
        *self
    }
}

impl Trait for & &u64 {
    fn trait_function() -> u64 {
        64 * 2
    }

    fn trait_method(self) -> u64 {
        let ptr = asm(r: self) { r: raw_ptr };

        ptr.read::<raw_ptr>().read::<u64>() * 2
    }

    fn trait_method_deref(self) -> u64 {
        **self * 2
    }
}

impl Trait for &mut &mut u64 {
    fn trait_function() -> u64 {
        64 * 2
    }

    fn trait_method(self) -> u64 {
        let ptr = asm(r: self) { r: raw_ptr };

        ptr.read::<raw_ptr>().read::<u64>() * 2
    }

    fn trait_method_deref(self) -> u64 {
        **self * 2
    }
}

type RefToU64 = &u64;
type RefToRefToU64 = & &u64;
type RefToRefToU64Alias = &RefToU64;

type RefToMutU64 = &mut u64;
type RefToMutRefToMutU64 = &mut &mut u64;
type RefToMutRefToMutU64Alias = &mut RefToMutU64;

fn test_references_to_mutable_values() -> u64 {
    let mut x = 123u64;
    let r_x = &mut x;
    let r_r_x = &mut &mut x;

    assert_eq(r_x.get_value(), x);
    assert_eq(r_x.mut_get_value(), x);
    assert_eq(r_x.get_value_deref(), x);
    assert_eq(r_x.mut_get_value_deref(), x);
    assert_eq(r_r_x.get_value(), x * 2);
    assert_eq(r_r_x.mut_get_value(), x * 2);
    assert_eq(r_r_x.get_value_deref(), x * 2);
    assert_eq(r_r_x.mut_get_value_deref(), x * 2);

    x = 2 * x;

    assert_eq(r_x.get_value(), x);
    assert_eq(r_x.mut_get_value(), x);
    assert_eq(r_x.get_value_deref(), x);
    assert_eq(r_x.mut_get_value_deref(), x);
    assert_eq(r_r_x.get_value(), x * 2);
    assert_eq(r_r_x.mut_get_value(), x * 2);
    assert_eq(r_r_x.get_value_deref(), x * 2);
    assert_eq(r_r_x.mut_get_value_deref(), x * 2);

    let mut array = [x, x + 100];
    let r_array = &mut array;
    let r_r_array = &mut &mut array;

    assert_eq(r_array.get_value(0), x);
    assert_eq(r_array.mut_get_value(0), x);
    assert_eq(r_array.get_value_deref(0), x);
    assert_eq(r_array.mut_get_value_deref(0), x);
    assert_eq(r_array.get_value(1), x + 100);
    assert_eq(r_array.mut_get_value(1), x + 100);
    assert_eq(r_array.get_value_deref(1), x + 100);
    assert_eq(r_array.mut_get_value_deref(1), x + 100);
    assert_eq(r_array.get_value_deref_array(1), x + 100);
    assert_eq(r_array.mut_get_value_deref_array(1), x + 100);

    assert_eq(r_r_array.get_value(0), x * 2);
    assert_eq(r_r_array.mut_get_value(0), x * 2);
    assert_eq(r_r_array.get_value_deref(0), x * 2);
    assert_eq(r_r_array.mut_get_value_deref(0), x * 2);
    assert_eq(r_r_array.get_value(1), (x + 100) * 2);
    assert_eq(r_r_array.mut_get_value(1), (x + 100) * 2);
    assert_eq(r_r_array.get_value_deref(1), (x + 100) * 2);
    assert_eq(r_r_array.mut_get_value_deref(1), (x + 100) * 2);
    assert_eq(r_r_array.get_value_deref_array(1), (x + 100) * 2);
    assert_eq(r_r_array.mut_get_value_deref_array(1), (x + 100) * 2);

    x = 2 * x;
    array[0] = x;
    array[1] = x + 100;

    assert_eq(r_array.get_value(0), x);
    assert_eq(r_array.mut_get_value(0), x);
    assert_eq(r_array.get_value_deref(0), x);
    assert_eq(r_array.mut_get_value_deref(0), x);
    assert_eq(r_array.get_value(1), x + 100);
    assert_eq(r_array.mut_get_value(1), x + 100);
    assert_eq(r_array.get_value_deref(1), x + 100);
    assert_eq(r_array.mut_get_value_deref(1), x + 100);
    assert_eq(r_array.get_value_deref_array(1), x + 100);
    assert_eq(r_array.mut_get_value_deref_array(1), x + 100);

    assert_eq(r_r_array.get_value(0), x * 2);
    assert_eq(r_r_array.mut_get_value(0), x * 2);
    assert_eq(r_r_array.get_value_deref(0), x * 2);
    assert_eq(r_r_array.mut_get_value_deref(0), x * 2);
    assert_eq(r_r_array.get_value(1), (x + 100) * 2);
    assert_eq(r_r_array.mut_get_value(1), (x + 100) * 2);
    assert_eq(r_r_array.get_value_deref(1), (x + 100) * 2);
    assert_eq(r_r_array.mut_get_value_deref(1), (x + 100) * 2);
    assert_eq(r_r_array.get_value_deref_array(1), (x + 100) * 2);
    assert_eq(r_r_array.mut_get_value_deref_array(1), (x + 100) * 2);

    assert_eq(r_x.trait_method(), x);
    assert_eq(r_x.trait_method_deref(), x);
    assert_eq(RefToU64::trait_function(), 64);
    assert_eq(RefToMutU64::trait_function(), 64);

    assert_eq(r_r_x.trait_method(), x * 2);
    assert_eq(r_r_x.trait_method_deref(), x * 2);
    assert_eq(RefToRefToU64::trait_function(), 64 * 2);
    assert_eq(RefToMutRefToMutU64::trait_function(), 64 * 2);
    assert_eq(RefToMutRefToMutU64Alias::trait_function(), 64 * 2);

    42
}

fn test_references() -> u64 {
    let mut x = 123u64;
    let r_x = &x;
    let r_r_x = & &x;

    assert_eq(r_x.get_value(), x);
    assert_eq(r_x.get_value_deref(), x);
    assert_eq(r_r_x.get_value(), x * 2);
    assert_eq(r_r_x.get_value_deref(), x * 2);

    x = 2 * x;

    assert_eq(r_x.get_value(), x);
    assert_eq(r_x.get_value_deref(), x);
    assert_eq(r_r_x.get_value(), x * 2);
    assert_eq(r_r_x.get_value_deref(), x * 2);

    let mut array = [x, x + 100];
    let r_array = &array;
    let r_r_array = & &array;

    assert_eq(r_array.get_value(0), x);
    assert_eq(r_array.get_value_deref(0), x);
    assert_eq(r_array.get_value(1), x + 100);
    assert_eq(r_array.get_value_deref(1), x + 100);
    assert_eq(r_array.get_value_deref_array(1), x + 100);

    assert_eq(r_r_array.get_value(0), x * 2);
    assert_eq(r_r_array.get_value_deref(0), x * 2);
    assert_eq(r_r_array.get_value(1), (x + 100) * 2);
    assert_eq(r_r_array.get_value_deref_array(1), (x + 100) * 2);

    x = 2 * x;
    array[0] = x;
    array[1] = x + 100;

    assert_eq(r_array.get_value(0), x);
    assert_eq(r_array.get_value_deref(0), x);
    assert_eq(r_array.get_value(1), x + 100);
    assert_eq(r_array.get_value_deref(1), x + 100);
    assert_eq(r_array.get_value_deref_array(1), x + 100);

    assert_eq(r_r_array.get_value(0), x * 2);
    assert_eq(r_r_array.get_value_deref(0), x * 2);
    assert_eq(r_r_array.get_value(1), (x + 100) * 2);
    assert_eq(r_r_array.get_value_deref_array(1), (x + 100) * 2);

    assert_eq(r_x.trait_method(), x);
    assert_eq(r_x.trait_method_deref(), x);
    assert_eq(RefToU64::trait_function(), 64);

    assert_eq(r_r_x.trait_method(), x * 2);
    assert_eq(r_r_x.trait_method_deref(), x * 2);
    assert_eq(RefToRefToU64::trait_function(), 64 * 2);
    assert_eq(RefToRefToU64Alias::trait_function(), 64 * 2);

    42
}

pub fn test() -> u64 {
    assert_eq(test_references(), 42);
    assert_eq(test_references_to_mutable_values(), 42);

    42
}
