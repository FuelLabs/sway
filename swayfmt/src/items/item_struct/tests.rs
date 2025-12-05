use forc_diagnostic::{println_green, println_red};
use paste::paste;
use prettydiff::{basic::DiffOp, diff_lines};
use test_macros::fmt_test_item;

fmt_test_item!(  annotated_struct
"pub struct Annotated {
    #[storage(write)]
    foo: u32,
    #[storage(read)]
    bar: String,
}",
            intermediate_whitespace
"pub struct Annotated{
                #[   storage(write  )]\n
                foo    : u32,
                #[   storage(read  )   ]
                bar   : String,
            }"
);

fmt_test_item!(  struct_with_where_clause
"pub struct HasWhereClause<T, A>
where
    T: Something,
    A: Something,
{
    t: T,
    a: A,
}",
            intermediate_whitespace
"pub  struct  HasWhereClause <  T, A >
   where
     T: Something,
    A :   Something,
 {
    t  : T,
    a : A,
} "
);
