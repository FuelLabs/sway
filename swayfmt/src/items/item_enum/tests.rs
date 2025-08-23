use forc_tracing::{println_green, println_red};
use pastey::paste;
use prettydiff::{basic::DiffOp, diff_lines};
use test_macros::fmt_test_item;

fmt_test_item!(  annotated_enum
"pub enum Annotated {
    #[storage(write)]
    foo: (),
    #[storage(read)]
    bar: (),
}",
            intermediate_whitespace
"pub enum Annotated{
                #[   storage(write  )]\n
                foo    : (),
                #[   storage(read  )   ]
                bar   : (),
            }"
);
