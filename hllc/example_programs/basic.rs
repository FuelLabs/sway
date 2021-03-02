contract {
    trait my_trait <T>{ 
      fn someFnThing(): T  
    } {
      fn some_other_trait_thing(): bool {
         true
      }
    }

    impl my_trait<u32> for my_struct {
        fn someFnThing() : u32 {
            return 5;
        }

    }

    pub fn contract_func_1<T>(a: T, y: u32): T {
      println("Test function.");
      let mut z: u8 = y;
      let x: byte = {
          // a code block w/ implicit return
          let y = 0b11110000;
          y
      };

      for x in 0..10 {
            z = z + 1 / 3;
      }

      let example_variable_decl = 5;
      let y = if true { 
            let x = 5;
            let z = 2;
            a
      } else { a };

      // should be an error since ther3's no return here
    }

    struct my_struct {
        FieldName: u64
    }
}

