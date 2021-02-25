contract {
    trait MyTrait { 
      fn some_fn_thing(): u32
    } {
      fn some_other_trait_thing(): bool {
         return true;
      }
    }

    struct my_struct {
        FieldName: u64
    }

    fn contract_func_1(x: u32, y: u32): bool {
      println("Test function.");
      let z = x.a.b.c;
      let x: byte = {
          // a code block w/ implicit return
          let x = 0b11110000;
          x
      };
      let example_variable_decl = 5;
      return example_variable_decl;
    }
}
