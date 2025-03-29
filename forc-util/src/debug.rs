use std::fmt;

macro_rules! debug_tuple_fields {
    ($dbg:ident, $self:ident, $($idx:tt),*) => {
        $(
            $dbg.field(&$self.$idx);
        )*
    };
}

pub fn debug_tuple<T: fmt::Debug>(tuple: &T, f: &mut fmt::Formatter) {
    let mut dbg = f.debug_tuple("");
    dbg.field(tuple);
    dbg.finish();
}
