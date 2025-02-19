script;

struct S1 {
   x: u64,
   y: u64,
}

const TEST: S1 = S1 { x: 101, y: 111 };

fn reference_to_int(ptr: &u64) -> u64 {
   asm(ptr:ptr) {
        ptr: u64
   }
}

#[inline(never)]
fn get_addr_x() -> u64 {
    let x_addr = &TEST.x;
    reference_to_int(x_addr)
}

#[inline(never)]
fn get_addr_y() -> u64 {
    let y_addr = &TEST.y;
    reference_to_int(y_addr)
}

#[inline(never)]
fn sum_x_y_addresses() -> u64 {
   get_addr_x() + get_addr_y()
}

fn main() -> u64 {
   0
}

#[test]
fn test_x_y_addr() {
   assert(get_addr_x() + get_addr_y() == sum_x_y_addresses());
}
