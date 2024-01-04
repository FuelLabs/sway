script;

impl &u64 {
    fn get_value(self) -> u64 {
        let ptr = asm(r: self) { r: raw_ptr };

        ptr.read::<u64>()
    }
    
    fn get_value_deref(self) -> u64 {
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
        assert(**self * 2 == 2 * **self);
        assert(* * self * 2 == 2 * * * self);
        assert(**self*2 == 2* **self);
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
}

impl & &[u64;2] {
    fn get_value(self, index: u64) -> u64 {
        let ptr = asm(r: self) { r: raw_ptr };

        ptr.read::<raw_ptr>().read::<[u64;2]>()[index] * 2
    }

    fn get_value_deref(self, index: u64) -> u64 {
        (**self)[index] * 2
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

type RefToU64 = &u64;
type RefToRefToU64 = & &u64;
type RefToRefToU64Alias = &RefToU64;

fn main() -> u64 {
    let mut x = 123u64;
    let r_x = &x;
    let r_r_x = & &x;

    assert(r_x.get_value() == x);
    assert(r_x.get_value_deref() == x);
    assert(r_r_x.get_value() == x * 2);
    assert(r_r_x.get_value_deref() == x * 2);

    x = 2 * x;

    assert(r_x.get_value() == x);
    assert(r_x.get_value_deref() == x);
    assert(r_r_x.get_value() == x * 2);
    assert(r_r_x.get_value_deref() == x * 2);

    let mut array = [x, x + 100];
    let r_array = &array;
    let r_r_array = & &array;

    assert(r_array.get_value(0) == x);
    assert(r_array.get_value_deref(0) == x);
    assert(r_array.get_value(1) == x + 100);
    assert(r_array.get_value_deref(1) == x + 100);

    assert(r_r_array.get_value(0) == x * 2);
    assert(r_r_array.get_value_deref(0) == x * 2);
    assert(r_r_array.get_value(1) == (x + 100) * 2);
    assert(r_r_array.get_value_deref(1) == (x + 100) * 2);

    x = 2 * x;
    array[0] = x;
    array[1] = x + 100;

    assert(r_array.get_value(0) == x);
    assert(r_array.get_value_deref(0) == x);
    assert(r_array.get_value(1) == x + 100);
    assert(r_array.get_value_deref(1) == x + 100);

    assert(r_r_array.get_value(0) == x * 2);
    assert(r_r_array.get_value_deref(0) == x * 2);
    assert(r_r_array.get_value(1) == (x + 100) * 2);
    assert(r_r_array.get_value_deref(1) == (x + 100) * 2);

    assert(r_x.trait_method() == x);
    assert(r_x.trait_method_deref() == x);
    assert(RefToU64::trait_function() == 64);

    assert(r_r_x.trait_method() == x * 2);
    assert(r_r_x.trait_method_deref() == x * 2);
    assert(RefToRefToU64::trait_function() == 64 * 2);
    assert(RefToRefToU64Alias::trait_function() == 64 * 2);
    
    42
}
