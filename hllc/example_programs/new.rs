predicate {
    trait GenericTrait  <T> where T: Add {
        fn a_tr_fn(a: T) -> T;
    }

    fn other_func(a: T) -> u32 {
            5
    }

    fn main() -> bool {
        let x = 5;
        let z = 5 + x;

        // todo: while loop
        while x < 5 {
            // todo: reassignment and check mutability
            x = x + 1;
        }
        
        let y = 5 - 2 / 1;
        true
    }
}

