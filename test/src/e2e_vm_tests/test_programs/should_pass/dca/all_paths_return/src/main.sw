script;

fn if_f() -> u64 {
    if true {
        return 0;
    } else {
        return 1;
    }
    // should trigger a warning
    return 2;
}

fn match_f() -> u64 {
  // TODO: Remove this return and get the match expression back once DCA supports __revert: https://github.com/FuelLabs/sway/issues/5214
  return 0;
   //match 10 {
   //  1 => return 8,
   //  _ => return 3,
   //}
   // should trigger a warning
   return 21;
}

fn main() -> u64 {
   if true {
      return if_f();
   } else {
     return match_f();
   }
}
