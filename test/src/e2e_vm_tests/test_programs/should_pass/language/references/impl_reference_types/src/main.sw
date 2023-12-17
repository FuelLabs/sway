script;

impl &u64 {
    fn get_value(self) -> u64 {
        let ptr = asm(r: self) { r: raw_ptr };

        ptr.read::<u64>()
    }
}

impl & &u64 {
    fn get_value(self) -> u64 {
        let ptr = asm(r: self) { r: raw_ptr };

        ptr.read::<raw_ptr>().read::<u64>() * 2
    }
}

impl &[u64;2] {
    fn get_value(self, index: u64) -> u64 {
        let ptr = asm(r: self) { r: raw_ptr };

        ptr.read::<[u64;2]>()[index]
    }
}

impl & &[u64;2] {
    fn get_value(self, index: u64) -> u64 {
        let ptr = asm(r: self) { r: raw_ptr };

        ptr.read::<raw_ptr>().read::<[u64;2]>()[index] * 2
    }
}

trait Trait {
    fn trait_function() -> u64;
    fn trait_method(self) -> u64;
}

impl Trait for &u64 {
    fn trait_function() -> u64 {
        64
    }

    fn trait_method(self) -> u64 {
        let ptr = asm(r: self) { r: raw_ptr };

        ptr.read::<u64>()
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
}

type RefToU64 = &u64;
type RefToRefToU64 = & &u64;
type RefToRefToU64Alias = &RefToU64;

fn main() -> u64 {
    let x = 123u64;
    let r_x = &x;
    let r_r_x = & &x;

    assert(r_x.get_value() == x);
    assert(r_r_x.get_value() == x * 2);

    let array = [x, x + 100];
    let r_array = &array;
    let r_r_array = & &array;

    assert(r_array.get_value(0) == x);
    assert(r_array.get_value(1) == x + 100);

    assert(r_r_array.get_value(0) == x * 2);
    assert(r_r_array.get_value(1) == (x + 100) * 2);

    assert(r_x.trait_method() == x);
    assert(RefToU64::trait_function() == 64);

    assert(r_r_x.trait_method() == x * 2);
    assert(RefToRefToU64::trait_function() == 64 * 2);
    assert(RefToRefToU64Alias::trait_function() == 64 * 2);
    
    42
}
