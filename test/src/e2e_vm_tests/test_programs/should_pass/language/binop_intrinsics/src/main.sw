script;

fn main() -> u64 {

  let a: u8 = 2;
  let b: u8 = 22;
  assert(__add(a, b) == 24);
  assert(__sub(b, a) == 20);
  assert(__mul(a, b) == 44);
  assert(__div(b, a) == 11);
  assert(__and(a, b) == 2);
  assert(__or(a, b) == 22);
  assert(__xor(a, b) == 20);

  let a: u16 = 22;
  let b: u16 = 44;
  assert(__add(a, b) == 66);
  assert(__sub(b, a) == 22);
  assert(__mul(a, b) == 968);
  assert(__div(b, a) == 2);
  assert(__and(a, b) == 4);  
  assert(__or(a, b) == 62);
  assert(__xor(a, b) == 58);

  let a: u32 = 22;
  let b: u32 = 44;
  assert(__add(a, b) == 66);
  assert(__sub(b, a) == 22);
  assert(__mul(a, b) == 968);
  assert(__div(b, a) == 2);
  assert(__and(a, b) == 4);
  assert(__or(a, b) == 62);
  assert(__xor(a, b) == 58);

  let a: u64 = 22;
  let b: u64 = 44;
  assert(__add(a, b) == 66);
  assert(__sub(b, a) == 22);
  assert(__mul(a, b) == 968);
  assert(__div(b, a) == 2);
  assert(__and(a, b) == 4);
  assert(__or(a, b) == 62);
  assert(__xor(a, b) == 58);

  assert(__xor(15, (__or(8, __and(5, 11)))) == 6);
  assert(__gt(2, 1) && __lt(1, 2));

  assert(__mod(0, 3) == 0);
  assert(__mod(1, 3) == 1);
  assert(__mod(2, 3) == 2);
  assert(__mod(3, 3) == 0);
  assert(__mod(4, 3) == 1);
  assert(__mod(5, 3) == 2);
  assert(__mod(6, 3) == 0);

  assert(__lsh(2, 3) == 16);
  assert(__rsh(16, 3) == 2);
  assert(__rsh(1, 1) == 0);

  2
}
