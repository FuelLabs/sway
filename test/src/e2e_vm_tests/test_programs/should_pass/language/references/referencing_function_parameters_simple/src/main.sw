script;

struct S1 {
   x: u64,
   y: u64,
}


fn main() -> u64 {
   0
}

#[inline(never)]
fn test_aggr(s: S1) {
   let s_addr = &s;
   let s_addr_u64 =
       asm(s_addr: s_addr) {
           s_addr: u64
       }
   ;
   log(s_addr_u64);
}

#[inline(never)]
fn test_int(s: u64) {
   let s_addr = &s;
   let s_addr_u64 =
       asm(s_addr: s_addr) {
           s_addr: u64
       }
   ;
   log(s_addr_u64);
   log(s);
}

#[test]
fn test_arg_addr() {
   let s = S1 { x: 1, y: 11 };
   test_aggr(s);
   let s = 23;
   test_int(s);
}
