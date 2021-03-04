predicate {
    trait GenericTrait  <T> where T: Add {
        fn a_tr_fn(a: T): T;
    }
    fn main(): bool {
        let x = 5;
        let z = 5 + x;
        
        let y = 5 - 2 / 1;
        true
    }
}

