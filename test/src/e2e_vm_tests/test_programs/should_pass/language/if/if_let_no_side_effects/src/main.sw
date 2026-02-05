// Test that proves that this issue is fixed: https://github.com/FuelLabs/sway/issues/5173
script;
 
struct Struct {
  x: u64,
  y: u64,
  z: u64
}
 
fn test_inc(ref mut i: u64) -> Struct {
    i = i + 11;
 
    Struct { x: 1111, y: 2222, z: 333 }
}
 
fn main() -> u64 {
    let mut i = 0;
 
    if let Struct { x, y, z: 0 } = test_inc(i) {
        let a = x + y;
        assert(a == 3333);
    };
    assert(i == 11);
 
    i
}