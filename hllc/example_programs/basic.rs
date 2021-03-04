contract {
    trait my_trait { 
      fn someFnThing() -> T  
    } {
      fn some_other_trait_thing() -> bool {
         true
      }
    }


    pub fn contract_func_1<T>(a: T, y: u32) ->T {
      println("Test function.", "other str", 3);
      let mut z: u8 = y;
      let x: u8 = {
          // a code block w/ implicit return
          let y = 0b11110000;
          y
      };


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

