script;

enum Either<Left, Right> {
  Left: Left,
  Right: Right,
}

fn main() -> u64 {
   let thing: Either<bool, u64> = Either::Left::<bool, u64>(true);

   if let Either::Right(n) = thing {
       n
   } else {
       0
   }
}
