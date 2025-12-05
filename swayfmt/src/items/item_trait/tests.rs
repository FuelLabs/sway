use forc_diagnostic::{println_green, println_red};
use paste::paste;
use prettydiff::{basic::DiffOp, diff_lines};
use test_macros::fmt_test_item;

fmt_test_item!(
trait_annotated_fn
"pub trait MyTrait {
    #[storage(read, write)]
    fn foo(self);
}",
intermediate_whitespace
"   pub   trait   MyTrait {
    #[storage(  read , write) ]
      
     fn foo(self);
}   "
);

fmt_test_item!(
trait_vertically_annotated_fn
"pub trait MyTrait {
    #[storage(read)]
    #[storage(write)]
    fn foo(self);
}",
intermediate_whitespace
"   pub   trait   MyTrait {
        #[storage(  read  ) ]
#[  storage(  write)]
      
     fn foo(self);
}   "
);

fmt_test_item!(
trait_commented_annotated_fn
"pub trait MyTrait {
    /// Doc
    /// Comment
    #[storage(read, write)]
    fn foo(self);
}",

intermediate_whitespace
"  pub   trait   MyTrait {
        /// Doc
/// Comment
    #[storage(  read , write) ]
      
     fn foo(self);
}   "
);

fmt_test_item!(
trait_commented_fn
"pub trait MyTrait {
    /// Comment
    fn foo(self);
}",

intermediate_whitespace
"  pub   trait   MyTrait {
/// Comment
      
     fn foo(self);
}   "
);

fmt_test_item!(trait_contains_const
"trait ConstantId {
    const ID: u32 = 1;
}",
intermediate_whitespace
"trait ConstantId {
    const    ID: u32 = 1;
}");

fmt_test_item!(
trait_normal_comment_two_fns
"pub trait MyTrait {
    // Before A
    fn a(self);
    // Before b
    fn b(self);
}",
intermediate_whitespace
"  pub   trait   MyTrait {
    // Before A
         fn a(self);
    // Before b
         fn b(self);
    }   "
);
fmt_test_item!(trait_multiline_method
"trait MyComplexTrait {
    fn complex_function(
        arg1: MyStruct<[b256; 3], u8>,
        arg2: [MyStruct<u64, bool>; 4],
        arg3: (str[5], bool),
        arg4: MyOtherStruct,
    ) -> str[6];
}",
intermediate_whitespace
"trait MyComplexTrait {
    fn complex_function(    arg1: MyStruct<[b256;  3], u8> , arg2: [MyStruct <u64, bool>; 4], arg3: ( str[5], bool ), arg4: MyOtherStruct)    -> str[6] ;
}");
fmt_test_item!(trait_with_where_clause
"trait TraitWithWhere<T, A>
where
    T: Something,
    A: Something,
{
    fn my_fn();
}",
intermediate_whitespace
"trait TraitWithWhere<T, A> where T: Something, A: Something { fn my_fn(); }");
