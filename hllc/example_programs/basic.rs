contract {
    /*
    trait MyTrait { 
      fn some_fn_thing(): u32
    } {
      fn some_other_trait_thing(): bool {
         return true;
      }
    }
    */

    fn contract_func_1<T>(x: u32, y: u32): T {
      println("Test function.");
      let z = x.a.b.c;
      let x: byte = {
          // a code block w/ implicit return
          let y = 0b11110000;
          y
      };
      let example_variable_decl = 5;
      return example_variable_decl;
    }

    /*
    struct my_struct {
        FieldName: u64
    }
    */
}
