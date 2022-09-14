pub use sway_ir_macros::*;
use {crate::Context, std::fmt};

pub struct WithContext<'a, 'c, T: ?Sized> {
    thing: &'a T,
    context: &'c Context,
}

pub trait DebugWithContext {
    fn fmt_with_context(&self, formatter: &mut fmt::Formatter, context: &Context) -> fmt::Result;

    fn with_context<'a, 'c>(&'a self, context: &'c Context) -> WithContext<'a, 'c, Self> {
        WithContext {
            thing: self,
            context,
        }
    }
}

impl<'t, T> DebugWithContext for &'t T
where
    T: fmt::Debug,
{
    fn fmt_with_context(&self, formatter: &mut fmt::Formatter, _context: &Context) -> fmt::Result {
        fmt::Debug::fmt(self, formatter)
    }
}

impl<'a, 'c, T> fmt::Debug for WithContext<'a, 'c, T>
where
    T: DebugWithContext,
{
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let WithContext { thing, context } = self;
        (*thing).fmt_with_context(formatter, context)
    }
}

impl<T> DebugWithContext for Vec<T>
where
    T: DebugWithContext,
{
    fn fmt_with_context(&self, formatter: &mut fmt::Formatter, context: &Context) -> fmt::Result {
        formatter
            .debug_list()
            .entries(self.iter().map(|value| (*value).with_context(context)))
            .finish()
    }
}

impl<T> DebugWithContext for [T]
where
    T: DebugWithContext,
{
    fn fmt_with_context(&self, formatter: &mut fmt::Formatter, context: &Context) -> fmt::Result {
        formatter
            .debug_list()
            .entries(self.iter().map(|value| (*value).with_context(context)))
            .finish()
    }
}

impl<T> DebugWithContext for Option<T>
where
    T: DebugWithContext,
{
    fn fmt_with_context(&self, formatter: &mut fmt::Formatter, context: &Context) -> fmt::Result {
        match self {
            Some(value) => formatter
                .debug_tuple("Some")
                .field(&(*value).with_context(context))
                .finish(),
            None => formatter.write_str("None"),
        }
    }
}

macro_rules! tuple_impl (
    ($($ty:ident,)*) => {
        impl<$($ty,)*> DebugWithContext for ($($ty,)*)
        where
            $($ty: DebugWithContext,)*
        {
            #[allow(unused_mut)]
            #[allow(unused_variables)]
            #[allow(non_snake_case)]
            fn fmt_with_context(&self, formatter: &mut fmt::Formatter, context: &Context) -> fmt::Result {
                let ($($ty,)*) = self;
                let mut debug_tuple = &mut formatter.debug_tuple("");
                $(
                    debug_tuple = debug_tuple.field(&(*$ty).with_context(context));
                )*
                debug_tuple.finish()
            }
        }
    };
);

macro_rules! tuple_impls (
    () => {
        tuple_impl!();
    };
    ($head:ident, $($tail:ident,)*) => {
        tuple_impls!($($tail,)*);
        tuple_impl!($head, $($tail,)*);
    };
);

tuple_impls!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15,);
