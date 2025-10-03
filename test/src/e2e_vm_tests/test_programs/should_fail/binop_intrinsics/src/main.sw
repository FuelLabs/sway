library;

struct A {
    a: u64,
}

pub fn main() {
   let _ = __add(A { a: 32 }, 32);
   let _ = __add("Hello", 22);
   let _ = __add("Hello", "Hello");
   let _ = __add(false, true);
   let _ = __add(1u32, 1u64);
   let _ = __add::<u32>(0, 1);

   let _ = __rsh("Hello", 1);
   let _ = __rsh(1, "Hello");
}
