#![allow(unused)]

use proc_macro2::{TokenStream, TokenTree};
use quote::ToTokens;
use std::{collections::hash_map::HashMap, fmt::Write};
use syn::{parse_quote, ItemMod};

// Ported from https://github.com/rust-lang/rust/blob/master/library/std/src/keyword_docs.rs

/// Documentation for sway keywords.
/// Primarily used for showing documentation on LSP hover requests.
#[derive(Debug)]
pub struct KeywordDocs {
    /// Key = keyword
    /// Value = documentation
    inner: HashMap<String, String>,
}

impl KeywordDocs {
    pub fn new() -> Self {
        let pub_keyword: ItemMod = parse_quote! {
            /// Make an item visible to others.
            ///
            /// The keyword `pub` makes any module, function, or data structure accessible from inside
            /// of external modules. The `pub` keyword may also be used in a `use` declaration to re-export
            /// an identifier from a namespace.
            ///
            /// For more information on the `pub` keyword, please see the visibility section
            /// of the [reference] and for some examples, see [Rust by Example].
            ///
            /// [reference]:../reference/visibility-and-privacy.html?highlight=pub#visibility-and-privacy
            /// [Rust by Example]:../rust-by-example/mod/visibility.html
            mod pub_keyword {}
        };

        let use_keyword: ItemMod = parse_quote! {
            /// Import or rename items from other crates or modules.
            ///
            /// Usually a `use` keyword is used to shorten the path required to refer to a module item.
            /// The keyword may appear in modules, blocks and even functions, usually at the top.
            ///
            /// The most basic usage of the keyword is `use path::to::item;`,
            /// though a number of convenient shortcuts are supported:
            ///
            ///   * Simultaneously binding a list of paths with a common prefix,
            ///     using the glob-like brace syntax `use a::b::{c, d, e::f, g::h::i};`
            ///   * Simultaneously binding a list of paths with a common prefix and their common parent module,
            ///     using the [`self`] keyword, such as `use a::b::{self, c, d::e};`
            ///   * Rebinding the target name as a new local name, using the syntax `use p::q::r as x;`.
            ///     This can also be used with the last two features: `use a::b::{self as ab, c as abc}`.
            ///   * Binding all paths matching a given prefix,
            ///     using the asterisk wildcard syntax `use a::b::*;`.
            ///   * Nesting groups of the previous features multiple times,
            ///     such as `use a::b::{self as ab, c, d::{*, e::f}};`
            ///   * Reexporting with visibility modifiers such as `pub use a::b;`
            ///   * Importing with `_` to only import the methods of a trait without binding it to a name
            ///     (to avoid conflict for example): `use ::std::io::Read as _;`.
            ///
            /// Using path qualifiers like [`crate`], [`super`] or [`self`] is supported: `use crate::a::b;`.
            ///
            /// Note that when the wildcard `*` is used on a type, it does not import its methods (though
            /// for `enum`s it imports the variants, as shown in the example below).
            ///
            /// ```compile_fail,edition2018
            /// enum ExampleEnum {
            ///     VariantA,
            ///     VariantB,
            /// }
            ///
            /// impl ExampleEnum {
            ///     fn new() -> Self {
            ///         Self::VariantA
            ///     }
            /// }
            ///
            /// use ExampleEnum::*;
            ///
            /// // Compiles.
            /// let _ = VariantA;
            ///
            /// // Does not compile !
            /// let n = new();
            /// ```
            ///
            /// For more information on `use` and paths in general, see the [Reference].
            ///
            /// The differences about paths and the `use` keyword between the 2015 and 2018 editions
            /// can also be found in the [Reference].
            ///
            /// [`crate`]: keyword.crate.html
            /// [`self`]: keyword.self.html
            /// [`super`]: keyword.super.html
            /// [Reference]: ../reference/items/use-declarations.html
            mod use_keyword {}
        };

        let as_keyword: ItemMod = parse_quote! {
            /// Cast between types, or rename an import.
            ///
            /// `as` is most commonly used to turn primitive types into other primitive types, but it has other
            /// uses that include turning pointers into addresses, addresses into pointers, and pointers into
            /// other pointers.
            ///
            /// ```sway
            /// let thing1: u8 = 89.0 as u8;
            /// assert_eq!('B' as u32, 66);
            /// assert_eq!(thing1 as char, 'Y');
            /// let thing2: f32 = thing1 as f32 + 10.5;
            /// assert_eq!(true as u8 + thing2 as u8, 100);
            /// ```
            ///
            /// In general, any cast that can be performed via ascribing the type can also be done using `as`,
            /// so instead of writing `let x: u32 = 123`, you can write `let x = 123 as u32` (note: `let x: u32
            /// = 123` would be best in that situation). The same is not true in the other direction, however;
            /// explicitly using `as` allows a few more coercions that aren't allowed implicitly, such as
            /// changing the type of a raw pointer or turning closures into raw pointers.
            ///
            /// `as` can be seen as the primitive for `From` and `Into`: `as` only works  with primitives
            /// (`u8`, `bool`, `str`, pointers, ...) whereas `From` and `Into`  also works with types like
            /// `String` or `Vec`.
            ///
            /// `as` can also be used with the `_` placeholder when the destination type can be inferred. Note
            /// that this can cause inference breakage and usually such code should use an explicit type for
            /// both clarity and stability. This is most useful when converting pointers using `as *const _` or
            /// `as *mut _` though the [`cast`][const-cast] method is recommended over `as *const _` and it is
            /// [the same][mut-cast] for `as *mut _`: those methods make the intent clearer.
            ///
            /// `as` is also used to rename imports in [`use`] and [`extern crate`][`crate`] statements:
            ///
            /// ```
            /// # #[allow(unused_imports)]
            /// use std::{mem as memory, net as network};
            /// // Now you can use the names `memory` and `network` to refer to `std::mem` and `std::net`.
            /// ```
            /// For more information on what `as` is capable of, see the [Reference].
            ///
            /// [Reference]: ../reference/expressions/operator-expr.html#type-cast-expressions
            /// [`crate`]: keyword.crate.html
            /// [`use`]: keyword.use.html
            /// [const-cast]: pointer::cast
            /// [mut-cast]: primitive.pointer.html#method.cast-1
            mod as_keyword {}
        };

        let struct_keyword: ItemMod = parse_quote! {
            /// A type that is composed of other types.
            ///
            /// Structs in Rust come in three flavors: Structs with named fields, tuple structs, and unit
            /// structs.
            ///
            /// ```rust
            /// struct Regular {
            ///     field1: f32,
            ///     field2: String,
            ///     pub field3: bool
            /// }
            ///
            /// struct Tuple(u32, String);
            ///
            /// struct Unit;
            /// ```
            ///
            /// Regular structs are the most commonly used. Each field defined within them has a name and a
            /// type, and once defined can be accessed using `example_struct.field` syntax. The fields of a
            /// struct share its mutability, so `foo.bar = 2;` would only be valid if `foo` was mutable. Adding
            /// `pub` to a field makes it visible to code in other modules, as well as allowing it to be
            /// directly accessed and modified.
            ///
            /// Tuple structs are similar to regular structs, but its fields have no names. They are used like
            /// tuples, with deconstruction possible via `let TupleStruct(x, y) = foo;` syntax. For accessing
            /// individual variables, the same syntax is used as with regular tuples, namely `foo.0`, `foo.1`,
            /// etc, starting at zero.
            ///
            /// Unit structs are most commonly used as marker. They have a size of zero bytes, but unlike empty
            /// enums they can be instantiated, making them isomorphic to the unit type `()`. Unit structs are
            /// useful when you need to implement a trait on something, but don't need to store any data inside
            /// it.
            ///
            /// # Instantiation
            ///
            /// Structs can be instantiated in different ways, all of which can be mixed and
            /// matched as needed. The most common way to make a new struct is via a constructor method such as
            /// `new()`, but when that isn't available (or you're writing the constructor itself), struct
            /// literal syntax is used:
            ///
            /// ```rust
            /// # struct Foo { field1: f32, field2: String, etc: bool }
            /// let example = Foo {
            ///     field1: 42.0,
            ///     field2: "blah".to_string(),
            ///     etc: true,
            /// };
            /// ```
            ///
            /// It's only possible to directly instantiate a struct using struct literal syntax when all of its
            /// fields are visible to you.
            ///
            /// There are a handful of shortcuts provided to make writing constructors more convenient, most
            /// common of which is the Field Init shorthand. When there is a variable and a field of the same
            /// name, the assignment can be simplified from `field: field` into simply `field`. The following
            /// example of a hypothetical constructor demonstrates this:
            ///
            /// ```rust
            /// struct User {
            ///     name: String,
            ///     admin: bool,
            /// }
            ///
            /// impl User {
            ///     pub fn new(name: String) -> Self {
            ///         Self {
            ///             name,
            ///             admin: false,
            ///         }
            ///     }
            /// }
            /// ```
            ///
            /// Another shortcut for struct instantiation is available, used when you need to make a new
            /// struct that has the same values as most of a previous struct of the same type, called struct
            /// update syntax:
            ///
            /// ```rust
            /// # struct Foo { field1: String, field2: () }
            /// # let thing = Foo { field1: "".to_string(), field2: () };
            /// let updated_thing = Foo {
            ///     field1: "a new value".to_string(),
            ///     ..thing
            /// };
            /// ```
            ///
            /// Tuple structs are instantiated in the same way as tuples themselves, except with the struct's
            /// name as a prefix: `Foo(123, false, 0.1)`.
            ///
            /// Empty structs are instantiated with just their name, and don't need anything else. `let thing =
            /// EmptyStruct;`
            ///
            /// # Style conventions
            ///
            /// Structs are always written in CamelCase, with few exceptions. While the trailing comma on a
            /// struct's list of fields can be omitted, it's usually kept for convenience in adding and
            /// removing fields down the line.
            ///
            /// For more information on structs, take a look at the [Rust Book][book] or the
            /// [Reference][reference].
            ///
            /// [`PhantomData`]: marker::PhantomData
            /// [book]: ../book/ch05-01-defining-structs.html
            /// [reference]: ../reference/items/structs.html
            mod struct_keyword {}
        };

        let enum_keyword: ItemMod = parse_quote! {
            /// A type that can be any one of several variants.
            ///
            /// Enums in Rust are similar to those of other compiled languages like C, but have important
            /// differences that make them considerably more powerful. What Rust calls enums are more commonly
            /// known as [Algebraic Data Types][ADT] if you're coming from a functional programming background.
            /// The important detail is that each enum variant can have data to go along with it.
            ///
            /// ```rust
            /// # struct Coord;
            /// enum SimpleEnum {
            ///     FirstVariant,
            ///     SecondVariant,
            ///     ThirdVariant,
            /// }
            ///
            /// enum Location {
            ///     Unknown,
            ///     Anonymous,
            ///     Known(Coord),
            /// }
            ///
            /// enum ComplexEnum {
            ///     Nothing,
            ///     Something(u32),
            ///     LotsOfThings {
            ///         usual_struct_stuff: bool,
            ///         blah: String,
            ///     }
            /// }
            ///
            /// enum EmptyEnum { }
            /// ```
            ///
            /// The first enum shown is the usual kind of enum you'd find in a C-style language. The second
            /// shows off a hypothetical example of something storing location data, with `Coord` being any
            /// other type that's needed, for example a struct. The third example demonstrates the kind of
            /// data a variant can store, ranging from nothing, to a tuple, to an anonymous struct.
            ///
            /// Instantiating enum variants involves explicitly using the enum's name as its namespace,
            /// followed by one of its variants. `SimpleEnum::SecondVariant` would be an example from above.
            /// When data follows along with a variant, such as with rust's built-in [`Option`] type, the data
            /// is added as the type describes, for example `Option::Some(123)`. The same follows with
            /// struct-like variants, with things looking like `ComplexEnum::LotsOfThings { usual_struct_stuff:
            /// true, blah: "hello!".to_string(), }`. Empty Enums are similar to [`!`] in that they cannot be
            /// instantiated at all, and are used mainly to mess with the type system in interesting ways.
            ///
            /// For more information, take a look at the [Rust Book] or the [Reference]
            ///
            /// [ADT]: https://en.wikipedia.org/wiki/Algebraic_data_type
            /// [Rust Book]: ../book/ch06-01-defining-an-enum.html
            /// [Reference]: ../reference/items/enumerations.html
            mod enum_keyword {}
        };

        let self_keyword: ItemMod = parse_quote! {
            /// The receiver of a method, or the current module.
            ///
            /// `self` is used in two situations: referencing the current module and marking
            /// the receiver of a method.
            ///
            /// In paths, `self` can be used to refer to the current module, either in a
            /// [`use`] statement or in a path to access an element:
            ///
            /// ```
            /// # #![allow(unused_imports)]
            /// use std::io::{self, Read};
            /// ```
            ///
            /// Is functionally the same as:
            ///
            /// ```
            /// # #![allow(unused_imports)]
            /// use std::io;
            /// use std::io::Read;
            /// ```
            ///
            /// Using `self` to access an element in the current module:
            ///
            /// ```
            /// # #![allow(dead_code)]
            /// # fn main() {}
            /// fn foo() {}
            /// fn bar() {
            ///     self::foo()
            /// }
            /// ```
            ///
            /// `self` as the current receiver for a method allows to omit the parameter
            /// type most of the time. With the exception of this particularity, `self` is
            /// used much like any other parameter:
            ///
            /// ```
            /// struct Foo(i32);
            ///
            /// impl Foo {
            ///     // No `self`.
            ///     fn new() -> Self {
            ///         Self(0)
            ///     }
            ///
            ///     // Consuming `self`.
            ///     fn consume(self) -> Self {
            ///         Self(self.0 + 1)
            ///     }
            ///
            ///     // Borrowing `self`.
            ///     fn borrow(&self) -> &i32 {
            ///         &self.0
            ///     }
            ///
            ///     // Borrowing `self` mutably.
            ///     fn borrow_mut(&mut self) -> &mut i32 {
            ///         &mut self.0
            ///     }
            /// }
            ///
            /// // This method must be called with a `Type::` prefix.
            /// let foo = Foo::new();
            /// assert_eq!(foo.0, 0);
            ///
            /// // Those two calls produces the same result.
            /// let foo = Foo::consume(foo);
            /// assert_eq!(foo.0, 1);
            /// let foo = foo.consume();
            /// assert_eq!(foo.0, 2);
            ///
            /// // Borrowing is handled automatically with the second syntax.
            /// let borrow_1 = Foo::borrow(&foo);
            /// let borrow_2 = foo.borrow();
            /// assert_eq!(borrow_1, borrow_2);
            ///
            /// // Borrowing mutably is handled automatically too with the second syntax.
            /// let mut foo = Foo::new();
            /// *Foo::borrow_mut(&mut foo) += 1;
            /// assert_eq!(foo.0, 1);
            /// *foo.borrow_mut() += 1;
            /// assert_eq!(foo.0, 2);
            /// ```
            ///
            /// Note that this automatic conversion when calling `foo.method()` is not
            /// limited to the examples above. See the [Reference] for more information.
            ///
            /// [`use`]: keyword.use.html
            /// [Reference]: ../reference/items/associated-items.html#methods
            mod self_keyword {}
        };

        let fn_keyword: ItemMod = parse_quote! {
            /// A function or function pointer.
            ///
            /// Functions are the primary way code is executed within Rust. Function blocks, usually just
            /// called functions, can be defined in a variety of different places and be assigned many
            /// different attributes and modifiers.
            ///
            /// Standalone functions that just sit within a module not attached to anything else are common,
            /// but most functions will end up being inside [`impl`] blocks, either on another type itself, or
            /// as a trait impl for that type.
            ///
            /// ```rust
            /// fn standalone_function() {
            ///     // code
            /// }
            ///
            /// pub fn public_thing(argument: bool) -> String {
            ///     // code
            ///     # "".to_string()
            /// }
            ///
            /// struct Thing {
            ///     foo: i32,
            /// }
            ///
            /// impl Thing {
            ///     pub fn new() -> Self {
            ///         Self {
            ///             foo: 42,
            ///         }
            ///     }
            /// }
            /// ```
            ///
            /// In addition to presenting fixed types in the form of `fn name(arg: type, ..) -> return_type`,
            /// functions can also declare a list of type parameters along with trait bounds that they fall
            /// into.
            ///
            /// ```rust
            /// fn generic_function<T: Clone>(x: T) -> (T, T, T) {
            ///     (x.clone(), x.clone(), x.clone())
            /// }
            ///
            /// fn generic_where<T>(x: T) -> T
            ///     where T: std::ops::Add<Output = T> + Copy
            /// {
            ///     x + x + x
            /// }
            /// ```
            ///
            /// Declaring trait bounds in the angle brackets is functionally identical to using a `where`
            /// clause. It's up to the programmer to decide which works better in each situation, but `where`
            /// tends to be better when things get longer than one line.
            ///
            /// Along with being made public via `pub`, `fn` can also have an [`extern`] added for use in
            /// FFI.
            ///
            /// For more information on the various types of functions and how they're used, consult the [Rust
            /// book] or the [Reference].
            ///
            /// [`impl`]: keyword.impl.html
            /// [`extern`]: keyword.extern.html
            /// [Rust book]: ../book/ch03-03-how-functions-work.html
            /// [Reference]: ../reference/items/functions.html
            mod fn_keyword {}
        };

        let trait_keyword: ItemMod = parse_quote! {
            /// A common interface for a group of types.
            ///
            /// A `trait` is like an interface that data types can implement. When a type
            /// implements a trait it can be treated abstractly as that trait using generics
            /// or trait objects.
            ///
            /// Traits can be made up of three varieties of associated items:
            ///
            /// - functions and methods
            /// - types
            /// - constants
            ///
            /// Traits may also contain additional type parameters. Those type parameters
            /// or the trait itself can be constrained by other traits.
            ///
            /// Traits can serve as markers or carry other logical semantics that
            /// aren't expressed through their items. When a type implements that
            /// trait it is promising to uphold its contract. [`Send`] and [`Sync`] are two
            /// such marker traits present in the standard library.
            ///
            /// See the [Reference][Ref-Traits] for a lot more information on traits.
            ///
            /// # Examples
            ///
            /// Traits are declared using the `trait` keyword. Types can implement them
            /// using [`impl`] `Trait` [`for`] `Type`:
            ///
            /// ```rust
            /// trait Zero {
            ///     const ZERO: Self;
            ///     fn is_zero(&self) -> bool;
            /// }
            ///
            /// impl Zero for i32 {
            ///     const ZERO: Self = 0;
            ///
            ///     fn is_zero(&self) -> bool {
            ///         *self == Self::ZERO
            ///     }
            /// }
            ///
            /// assert_eq!(i32::ZERO, 0);
            /// assert!(i32::ZERO.is_zero());
            /// assert!(!4.is_zero());
            /// ```
            ///
            /// With an associated type:
            ///
            /// ```rust
            /// trait Builder {
            ///     type Built;
            ///
            ///     fn build(&self) -> Self::Built;
            /// }
            /// ```
            ///
            /// Traits can be generic, with constraints or without:
            ///
            /// ```rust
            /// trait MaybeFrom<T> {
            ///     fn maybe_from(value: T) -> Option<Self>
            ///     where
            ///         Self: Sized;
            /// }
            /// ```
            ///
            /// Traits can build upon the requirements of other traits. In the example
            /// below `Iterator` is a **supertrait** and `ThreeIterator` is a **subtrait**:
            ///
            /// ```rust
            /// trait ThreeIterator: std::iter::Iterator {
            ///     fn next_three(&mut self) -> Option<[Self::Item; 3]>;
            /// }
            /// ```
            ///
            /// Traits can be used in functions, as parameters:
            ///
            /// ```rust
            /// # #![allow(dead_code)]
            /// fn debug_iter<I: Iterator>(it: I) where I::Item: std::fmt::Debug {
            ///     for elem in it {
            ///         println!("{elem:#?}");
            ///     }
            /// }
            ///
            /// // u8_len_1, u8_len_2 and u8_len_3 are equivalent
            ///
            /// fn u8_len_1(val: impl Into<Vec<u8>>) -> usize {
            ///     val.into().len()
            /// }
            ///
            /// fn u8_len_2<T: Into<Vec<u8>>>(val: T) -> usize {
            ///     val.into().len()
            /// }
            ///
            /// fn u8_len_3<T>(val: T) -> usize
            /// where
            ///     T: Into<Vec<u8>>,
            /// {
            ///     val.into().len()
            /// }
            /// ```
            ///
            /// Or as return types:
            ///
            /// ```rust
            /// # #![allow(dead_code)]
            /// fn from_zero_to(v: u8) -> impl Iterator<Item = u8> {
            ///     (0..v).into_iter()
            /// }
            /// ```
            ///
            /// The use of the [`impl`] keyword in this position allows the function writer
            /// to hide the concrete type as an implementation detail which can change
            /// without breaking user's code.
            ///
            /// # Trait objects
            ///
            /// A *trait object* is an opaque value of another type that implements a set of
            /// traits. A trait object implements all specified traits as well as their
            /// supertraits (if any).
            ///
            /// The syntax is the following: `dyn BaseTrait + AutoTrait1 + ... AutoTraitN`.
            /// Only one `BaseTrait` can be used so this will not compile:
            ///
            /// ```rust,compile_fail,E0225
            /// trait A {}
            /// trait B {}
            ///
            /// let _: Box<dyn A + B>;
            /// ```
            ///
            /// Neither will this, which is a syntax error:
            ///
            /// ```rust,compile_fail
            /// trait A {}
            /// trait B {}
            ///
            /// let _: Box<dyn A + dyn B>;
            /// ```
            ///
            /// On the other hand, this is correct:
            ///
            /// ```rust
            /// trait A {}
            ///
            /// let _: Box<dyn A + Send + Sync>;
            /// ```
            ///
            /// The [Reference][Ref-Trait-Objects] has more information about trait objects,
            /// their limitations and the differences between editions.
            ///
            /// # Unsafe traits
            ///
            /// Some traits may be unsafe to implement. Using the [`unsafe`] keyword in
            /// front of the trait's declaration is used to mark this:
            ///
            /// ```rust
            /// unsafe trait UnsafeTrait {}
            ///
            /// unsafe impl UnsafeTrait for i32 {}
            /// ```
            ///
            /// # Differences between the 2015 and 2018 editions
            ///
            /// In the 2015 edition the parameters pattern was not needed for traits:
            ///
            /// ```rust,edition2015
            /// # #![allow(anonymous_parameters)]
            /// trait Tr {
            ///     fn f(i32);
            /// }
            /// ```
            ///
            /// This behavior is no longer valid in edition 2018.
            ///
            /// [`for`]: keyword.for.html
            /// [`impl`]: keyword.impl.html
            /// [`unsafe`]: keyword.unsafe.html
            /// [Ref-Traits]: ../reference/items/traits.html
            /// [Ref-Trait-Objects]: ../reference/types/trait-object.html
            mod trait_keyword {}
        };

        let impl_keyword: ItemMod = parse_quote! {
            /// Implement some functionality for a type.
            ///
            /// The `impl` keyword is primarily used to define implementations on types. Inherent
            /// implementations are standalone, while trait implementations are used to implement traits for
            /// types, or other traits.
            ///
            /// Functions and consts can both be defined in an implementation. A function defined in an
            /// `impl` block can be standalone, meaning it would be called like `Foo::bar()`. If the function
            /// takes `self`, `&self`, or `&mut self` as its first argument, it can also be called using
            /// method-call syntax, a familiar feature to any object oriented programmer, like `foo.bar()`.
            ///
            /// ```rust
            /// struct Example {
            ///     number: i32,
            /// }
            ///
            /// impl Example {
            ///     fn boo() {
            ///         println!("boo! Example::boo() was called!");
            ///     }
            ///
            ///     fn answer(&mut self) {
            ///         self.number += 42;
            ///     }
            ///
            ///     fn get_number(&self) -> i32 {
            ///         self.number
            ///     }
            /// }
            ///
            /// trait Thingy {
            ///     fn do_thingy(&self);
            /// }
            ///
            /// impl Thingy for Example {
            ///     fn do_thingy(&self) {
            ///         println!("doing a thing! also, number is {}!", self.number);
            ///     }
            /// }
            /// ```
            ///
            /// For more information on implementations, see the [Rust book][book1] or the [Reference].
            ///
            /// The other use of the `impl` keyword is in `impl Trait` syntax, which can be seen as a shorthand
            /// for "a concrete type that implements this trait". Its primary use is working with closures,
            /// which have type definitions generated at compile time that can't be simply typed out.
            ///
            /// ```rust
            /// fn thing_returning_closure() -> impl Fn(i32) -> bool {
            ///     println!("here's a closure for you!");
            ///     |x: i32| x % 3 == 0
            /// }
            /// ```
            ///
            /// For more information on `impl Trait` syntax, see the [Rust book][book2].
            ///
            /// [book1]: ../book/ch05-03-method-syntax.html
            /// [Reference]: ../reference/items/implementations.html
            /// [book2]: ../book/ch10-02-traits.html#returning-types-that-implement-traits
            mod impl_keyword {}
        };

        let for_keyword: ItemMod = parse_quote! {
            /// Iteration with [`in`], trait implementation with [`impl`], or [higher-ranked trait bounds]
            /// (`for<'a>`).
            ///
            /// The `for` keyword is used in many syntactic locations:
            ///
            /// * `for` is used in for-in-loops (see below).
            /// * `for` is used when implementing traits as in `impl Trait for Type` (see [`impl`] for more info
            ///   on that).
            /// * `for` is also used for [higher-ranked trait bounds] as in `for<'a> &'a T: PartialEq<i32>`.
            ///
            /// for-in-loops, or to be more precise, iterator loops, are a simple syntactic sugar over a common
            /// practice within Rust, which is to loop over anything that implements [`IntoIterator`] until the
            /// iterator returned by `.into_iter()` returns `None` (or the loop body uses `break`).
            ///
            /// ```rust
            /// for i in 0..5 {
            ///     println!("{}", i * 2);
            /// }
            ///
            /// for i in std::iter::repeat(5) {
            ///     println!("turns out {i} never stops being 5");
            ///     break; // would loop forever otherwise
            /// }
            ///
            /// 'outer: for x in 5..50 {
            ///     for y in 0..10 {
            ///         if x == y {
            ///             break 'outer;
            ///         }
            ///     }
            /// }
            /// ```
            ///
            /// As shown in the example above, `for` loops (along with all other loops) can be tagged, using
            /// similar syntax to lifetimes (only visually similar, entirely distinct in practice). Giving the
            /// same tag to `break` breaks the tagged loop, which is useful for inner loops. It is definitely
            /// not a goto.
            ///
            /// A `for` loop expands as shown:
            ///
            /// ```rust
            /// # fn code() { }
            /// # let iterator = 0..2;
            /// for loop_variable in iterator {
            ///     code()
            /// }
            /// ```
            ///
            /// ```rust
            /// # fn code() { }
            /// # let iterator = 0..2;
            /// {
            ///     let result = match IntoIterator::into_iter(iterator) {
            ///         mut iter => loop {
            ///             match iter.next() {
            ///                 None => break,
            ///                 Some(loop_variable) => { code(); },
            ///             };
            ///         },
            ///     };
            ///     result
            /// }
            /// ```
            ///
            /// More details on the functionality shown can be seen at the [`IntoIterator`] docs.
            ///
            /// For more information on for-loops, see the [Rust book] or the [Reference].
            ///
            /// See also, [`loop`], [`while`].
            ///
            /// [`in`]: keyword.in.html
            /// [`impl`]: keyword.impl.html
            /// [`loop`]: keyword.loop.html
            /// [`while`]: keyword.while.html
            /// [higher-ranked trait bounds]: ../reference/trait-bounds.html#higher-ranked-trait-bounds
            /// [Rust book]:
            /// ../book/ch03-05-control-flow.html#looping-through-a-collection-with-for
            /// [Reference]: ../reference/expressions/loop-expr.html#iterator-loops
            mod for_keyword {}
        };

        let const_keyword: ItemMod = parse_quote! {
            /// Compile-time constants, compile-time evaluable functions, and raw pointers.
            ///
            /// ## Compile-time constants
            ///
            /// Sometimes a certain value is used many times throughout a program, and it can become
            /// inconvenient to copy it over and over. What's more, it's not always possible or desirable to
            /// make it a variable that gets carried around to each function that needs it. In these cases, the
            /// `const` keyword provides a convenient alternative to code duplication:
            ///
            /// ```rust
            /// const THING: u32 = 0xABAD1DEA;
            ///
            /// let foo = 123 + THING;
            /// ```
            ///
            /// Constants must be explicitly typed; unlike with `let`, you can't ignore their type and let the
            /// compiler figure it out. Any constant value can be defined in a `const`, which in practice happens
            /// to be most things that would be reasonable to have in a constant (barring `const fn`s). For
            /// example, you can't have a [`File`] as a `const`.
            ///
            /// [`File`]: crate::fs::File
            ///
            /// The only lifetime allowed in a constant is `'static`, which is the lifetime that encompasses
            /// all others in a Rust program. For example, if you wanted to define a constant string, it would
            /// look like this:
            ///
            /// ```rust
            /// const WORDS: &'static str = "hello rust!";
            /// ```
            ///
            /// Thanks to static lifetime elision, you usually don't have to explicitly use `'static`:
            ///
            /// ```rust
            /// const WORDS: &str = "hello convenience!";
            /// ```
            ///
            /// `const` items looks remarkably similar to `static` items, which introduces some confusion as
            /// to which one should be used at which times. To put it simply, constants are inlined wherever
            /// they're used, making using them identical to simply replacing the name of the `const` with its
            /// value. Static variables, on the other hand, point to a single location in memory, which all
            /// accesses share. This means that, unlike with constants, they can't have destructors, and act as
            /// a single value across the entire codebase.
            ///
            /// Constants, like statics, should always be in `SCREAMING_SNAKE_CASE`.
            ///
            /// For more detail on `const`, see the [Rust Book] or the [Reference].
            ///
            /// ## Compile-time evaluable functions
            ///
            /// The other main use of the `const` keyword is in `const fn`. This marks a function as being
            /// callable in the body of a `const` or `static` item and in array initializers (commonly called
            /// "const contexts"). `const fn` are restricted in the set of operations they can perform, to
            /// ensure that they can be evaluated at compile-time. See the [Reference][const-eval] for more
            /// detail.
            ///
            /// Turning a `fn` into a `const fn` has no effect on run-time uses of that function.
            ///
            /// ## Other uses of `const`
            ///
            /// The `const` keyword is also used in raw pointers in combination with `mut`, as seen in `*const
            /// T` and `*mut T`. More about `const` as used in raw pointers can be read at the Rust docs for the [pointer primitive].
            ///
            /// [pointer primitive]: pointer
            /// [Rust Book]: ../book/ch03-01-variables-and-mutability.html#constants
            /// [Reference]: ../reference/items/constant-items.html
            /// [const-eval]: ../reference/const_eval.html
            mod const_keyword {}
        };

        let return_keyword: ItemMod = parse_quote! {
            /// Return a value from a function.
            ///
            /// A `return` marks the end of an execution path in a function:
            ///
            /// ```
            /// fn foo() -> i32 {
            ///     return 3;
            /// }
            /// assert_eq!(foo(), 3);
            /// ```
            ///
            /// `return` is not needed when the returned value is the last expression in the
            /// function. In this case the `;` is omitted:
            ///
            /// ```
            /// fn foo() -> i32 {
            ///     3
            /// }
            /// assert_eq!(foo(), 3);
            /// ```
            ///
            /// `return` returns from the function immediately (an "early return"):
            ///
            /// ```no_run
            /// use std::fs::File;
            /// use std::io::{Error, ErrorKind, Read, Result};
            ///
            /// fn main() -> Result<()> {
            ///     let mut file = match File::open("foo.txt") {
            ///         Ok(f) => f,
            ///         Err(e) => return Err(e),
            ///     };
            ///
            ///     let mut contents = String::new();
            ///     let size = match file.read_to_string(&mut contents) {
            ///         Ok(s) => s,
            ///         Err(e) => return Err(e),
            ///     };
            ///
            ///     if contents.contains("impossible!") {
            ///         return Err(Error::new(ErrorKind::Other, "oh no!"));
            ///     }
            ///
            ///     if size > 9000 {
            ///         return Err(Error::new(ErrorKind::Other, "over 9000!"));
            ///     }
            ///
            ///     assert_eq!(contents, "Hello, world!");
            ///     Ok(())
            /// }
            /// ```
            mod return_keyword {}
        };

        let if_keyword: ItemMod = parse_quote! {
            /// Evaluate a block if a condition holds.
            ///
            /// `if` is a familiar construct to most programmers, and is the main way you'll often do logic in
            /// your code. However, unlike in most languages, `if` blocks can also act as expressions.
            ///
            /// ```rust
            /// # let rude = true;
            /// if 1 == 2 {
            ///     println!("whoops, mathematics broke");
            /// } else {
            ///     println!("everything's fine!");
            /// }
            ///
            /// let greeting = if rude {
            ///     "sup nerd."
            /// } else {
            ///     "hello, friend!"
            /// };
            ///
            /// if let Ok(x) = "123".parse::<i32>() {
            ///     println!("{} double that and you get {}!", greeting, x * 2);
            /// }
            /// ```
            ///
            /// Shown above are the three typical forms an `if` block comes in. First is the usual kind of
            /// thing you'd see in many languages, with an optional `else` block. Second uses `if` as an
            /// expression, which is only possible if all branches return the same type. An `if` expression can
            /// be used everywhere you'd expect. The third kind of `if` block is an `if let` block, which
            /// behaves similarly to using a `match` expression:
            ///
            /// ```rust
            /// if let Some(x) = Some(123) {
            ///     // code
            ///     # let _ = x;
            /// } else {
            ///     // something else
            /// }
            ///
            /// match Some(123) {
            ///     Some(x) => {
            ///         // code
            ///         # let _ = x;
            ///     },
            ///     _ => {
            ///         // something else
            ///     },
            /// }
            /// ```
            ///
            /// Each kind of `if` expression can be mixed and matched as needed.
            ///
            /// ```rust
            /// if true == false {
            ///     println!("oh no");
            /// } else if "something" == "other thing" {
            ///     println!("oh dear");
            /// } else if let Some(200) = "blarg".parse::<i32>().ok() {
            ///     println!("uh oh");
            /// } else {
            ///     println!("phew, nothing's broken");
            /// }
            /// ```
            ///
            /// The `if` keyword is used in one other place in Rust, namely as a part of pattern matching
            /// itself, allowing patterns such as `Some(x) if x > 200` to be used.
            ///
            /// For more information on `if` expressions, see the [Rust book] or the [Reference].
            ///
            /// [Rust book]: ../book/ch03-05-control-flow.html#if-expressions
            /// [Reference]: ../reference/expressions/if-expr.html
            mod if_keyword {}
        };

        let else_keyword: ItemMod = parse_quote! {
            /// What expression to evaluate when an [`if`] condition evaluates to [`false`].
            ///
            /// `else` expressions are optional. When no else expressions are supplied it is assumed to evaluate
            /// to the unit type `()`.
            ///
            /// The type that the `else` blocks evaluate to must be compatible with the type that the `if` block
            /// evaluates to.
            ///
            /// As can be seen below, `else` must be followed by either: `if`, `if let`, or a block `{}` and it
            /// will return the value of that expression.
            ///
            /// ```rust
            /// let result = if true == false {
            ///     "oh no"
            /// } else if "something" == "other thing" {
            ///     "oh dear"
            /// } else if let Some(200) = "blarg".parse::<i32>().ok() {
            ///     "uh oh"
            /// } else {
            ///     println!("Sneaky side effect.");
            ///     "phew, nothing's broken"
            /// };
            /// ```
            ///
            /// Here's another example but here we do not try and return an expression:
            ///
            /// ```rust
            /// if true == false {
            ///     println!("oh no");
            /// } else if "something" == "other thing" {
            ///     println!("oh dear");
            /// } else if let Some(200) = "blarg".parse::<i32>().ok() {
            ///     println!("uh oh");
            /// } else {
            ///     println!("phew, nothing's broken");
            /// }
            /// ```
            ///
            /// The above is _still_ an expression but it will always evaluate to `()`.
            ///
            /// There is possibly no limit to the number of `else` blocks that could follow an `if` expression
            /// however if you have several then a [`match`] expression might be preferable.
            ///
            /// Read more about control flow in the [Rust Book].
            ///
            /// [Rust Book]: ../book/ch03-05-control-flow.html#handling-multiple-conditions-with-else-if
            /// [`match`]: keyword.match.html
            /// [`false`]: keyword.false.html
            /// [`if`]: keyword.if.html
            mod else_keyword {}
        };

        let match_keyword: ItemMod = parse_quote! {
            /// Control flow based on pattern matching.
            ///
            /// `match` can be used to run code conditionally. Every pattern must
            /// be handled exhaustively either explicitly or by using wildcards like
            /// `_` in the `match`. Since `match` is an expression, values can also be
            /// returned.
            ///
            /// ```rust
            /// let opt = Option::None::<usize>;
            /// let x = match opt {
            ///     Some(int) => int,
            ///     None => 10,
            /// };
            /// assert_eq!(x, 10);
            ///
            /// let a_number = Option::Some(10);
            /// match a_number {
            ///     Some(x) if x <= 5 => println!("0 to 5 num = {x}"),
            ///     Some(x @ 6..=10) => println!("6 to 10 num = {x}"),
            ///     None => panic!(),
            ///     // all other numbers
            ///     _ => panic!(),
            /// }
            /// ```
            ///
            /// `match` can be used to gain access to the inner members of an enum
            /// and use them directly.
            ///
            /// ```rust
            /// enum Outer {
            ///     Double(Option<u8>, Option<String>),
            ///     Single(Option<u8>),
            ///     Empty
            /// }
            ///
            /// let get_inner = Outer::Double(None, Some(String::new()));
            /// match get_inner {
            ///     Outer::Double(None, Some(st)) => println!("{st}"),
            ///     Outer::Single(opt) => println!("{opt:?}"),
            ///     _ => panic!(),
            /// }
            /// ```
            ///
            /// For more information on `match` and matching in general, see the [Reference].
            ///
            /// [Reference]: ../reference/expressions/match-expr.html
            mod match_keyword {}
        };

        let mut_keyword: ItemMod = parse_quote! {
            /// A mutable variable, reference, or pointer.
            ///
            /// `mut` can be used in several situations. The first is mutable variables,
            /// which can be used anywhere you can bind a value to a variable name. Some
            /// examples:
            ///
            /// ```rust
            /// // A mutable variable in the parameter list of a function.
            /// fn foo(mut x: u8, y: u8) -> u8 {
            ///     x += y;
            ///     x
            /// }
            ///
            /// // Modifying a mutable variable.
            /// # #[allow(unused_assignments)]
            /// let mut a = 5;
            /// a = 6;
            ///
            /// assert_eq!(foo(3, 4), 7);
            /// assert_eq!(a, 6);
            /// ```
            ///
            /// The second is mutable references. They can be created from `mut` variables
            /// and must be unique: no other variables can have a mutable reference, nor a
            /// shared reference.
            ///
            /// ```rust
            /// // Taking a mutable reference.
            /// fn push_two(v: &mut Vec<u8>) {
            ///     v.push(2);
            /// }
            ///
            /// // A mutable reference cannot be taken to a non-mutable variable.
            /// let mut v = vec![0, 1];
            /// // Passing a mutable reference.
            /// push_two(&mut v);
            ///
            /// assert_eq!(v, vec![0, 1, 2]);
            /// ```
            ///
            /// ```rust,compile_fail,E0502
            /// let mut v = vec![0, 1];
            /// let mut_ref_v = &mut v;
            /// ##[allow(unused)]
            /// let ref_v = &v;
            /// mut_ref_v.push(2);
            /// ```
            ///
            /// Mutable raw pointers work much like mutable references, with the added
            /// possibility of not pointing to a valid object. The syntax is `*mut Type`.
            ///
            /// More information on mutable references and pointers can be found in the [Reference].
            ///
            /// [Reference]: ../reference/types/pointer.html#mutable-references-mut
            mod mut_keyword {}
        };

        let let_keyword: ItemMod = parse_quote! {
            /// Bind a value to a variable.
            ///
            /// The primary use for the `let` keyword is in `let` statements, which are used to introduce a new
            /// set of variables into the current scope, as given by a pattern.
            ///
            /// ```rust
            /// # #![allow(unused_assignments)]
            /// let thing1: i32 = 100;
            /// let thing2 = 200 + thing1;
            ///
            /// let mut changing_thing = true;
            /// changing_thing = false;
            ///
            /// let (part1, part2) = ("first", "second");
            ///
            /// struct Example {
            ///     a: bool,
            ///     b: u64,
            /// }
            ///
            /// let Example { a, b: _ } = Example {
            ///     a: true,
            ///     b: 10004,
            /// };
            /// assert!(a);
            /// ```
            ///
            /// The pattern is most commonly a single variable, which means no pattern matching is done and
            /// the expression given is bound to the variable. Apart from that, patterns used in `let` bindings
            /// can be as complicated as needed, given that the pattern is exhaustive. See the [Rust
            /// book][book1] for more information on pattern matching. The type of the pattern is optionally
            /// given afterwards, but if left blank is automatically inferred by the compiler if possible.
            ///
            /// Variables in Rust are immutable by default, and require the `mut` keyword to be made mutable.
            ///
            /// Multiple variables can be defined with the same name, known as shadowing. This doesn't affect
            /// the original variable in any way beyond being unable to directly access it beyond the point of
            /// shadowing. It continues to remain in scope, getting dropped only when it falls out of scope.
            /// Shadowed variables don't need to have the same type as the variables shadowing them.
            ///
            /// ```rust
            /// let shadowing_example = true;
            /// let shadowing_example = 123.4;
            /// let shadowing_example = shadowing_example as u32;
            /// let mut shadowing_example = format!("cool! {shadowing_example}");
            /// shadowing_example += " something else!"; // not shadowing
            /// ```
            ///
            /// Other places the `let` keyword is used include along with [`if`], in the form of `if let`
            /// expressions. They're useful if the pattern being matched isn't exhaustive, such as with
            /// enumerations. `while let` also exists, which runs a loop with a pattern matched value until
            /// that pattern can't be matched.
            ///
            /// For more information on the `let` keyword, see the [Rust book][book2] or the [Reference]
            ///
            /// [book1]: ../book/ch06-02-match.html
            /// [`if`]: keyword.if.html
            /// [book2]: ../book/ch18-01-all-the-places-for-patterns.html#let-statements
            /// [Reference]: ../reference/statements.html#let-statements
            mod let_keyword {}
        };

        let while_keyword: ItemMod = parse_quote! {
            /// Loop while a condition is upheld.
            ///
            /// A `while` expression is used for predicate loops. The `while` expression runs the conditional
            /// expression before running the loop body, then runs the loop body if the conditional
            /// expression evaluates to `true`, or exits the loop otherwise.
            ///
            /// ```rust
            /// let mut counter = 0;
            ///
            /// while counter < 10 {
            ///     println!("{counter}");
            ///     counter += 1;
            /// }
            /// ```
            ///
            /// Like the [`for`] expression, we can use `break` and `continue`. A `while` expression
            /// cannot break with a value and always evaluates to `()` unlike [`loop`].
            ///
            /// ```rust
            /// let mut i = 1;
            ///
            /// while i < 100 {
            ///     i *= 2;
            ///     if i == 64 {
            ///         break; // Exit when `i` is 64.
            ///     }
            /// }
            /// ```
            ///
            /// As `if` expressions have their pattern matching variant in `if let`, so too do `while`
            /// expressions with `while let`. The `while let` expression matches the pattern against the
            /// expression, then runs the loop body if pattern matching succeeds, or exits the loop otherwise.
            /// We can use `break` and `continue` in `while let` expressions just like in `while`.
            ///
            /// ```rust
            /// let mut counter = Some(0);
            ///
            /// while let Some(i) = counter {
            ///     if i == 10 {
            ///         counter = None;
            ///     } else {
            ///         println!("{i}");
            ///         counter = Some (i + 1);
            ///     }
            /// }
            /// ```
            ///
            /// For more information on `while` and loops in general, see the [reference].
            ///
            /// See also, [`for`], [`loop`].
            ///
            /// [`for`]: keyword.for.html
            /// [`loop`]: keyword.loop.html
            /// [reference]: ../reference/expressions/loop-expr.html#predicate-loops
            mod while_keyword {}
        };

        let where_keyword: ItemMod = parse_quote! {
            /// Add constraints that must be upheld to use an item.
            ///
            /// `where` allows specifying constraints on lifetime and generic parameters.
            /// The [RFC] introducing `where` contains detailed information about the
            /// keyword.
            ///
            /// # Examples
            ///
            /// `where` can be used for constraints with traits:
            ///
            /// ```rust
            /// fn new<T: Default>() -> T {
            ///     T::default()
            /// }
            ///
            /// fn new_where<T>() -> T
            /// where
            ///     T: Default,
            /// {
            ///     T::default()
            /// }
            ///
            /// assert_eq!(0.0, new());
            /// assert_eq!(0.0, new_where());
            ///
            /// assert_eq!(0, new());
            /// assert_eq!(0, new_where());
            /// ```
            ///
            /// `where` can also be used for lifetimes.
            ///
            /// This compiles because `longer` outlives `shorter`, thus the constraint is
            /// respected:
            ///
            /// ```rust
            /// fn select<'short, 'long>(s1: &'short str, s2: &'long str, second: bool) -> &'short str
            /// where
            ///     'long: 'short,
            /// {
            ///     if second { s2 } else { s1 }
            /// }
            ///
            /// let outer = String::from("Long living ref");
            /// let longer = &outer;
            /// {
            ///     let inner = String::from("Short living ref");
            ///     let shorter = &inner;
            ///
            ///     assert_eq!(select(shorter, longer, false), shorter);
            ///     assert_eq!(select(shorter, longer, true), longer);
            /// }
            /// ```
            ///
            /// On the other hand, this will not compile because the `where 'b: 'a` clause
            /// is missing: the `'b` lifetime is not known to live at least as long as `'a`
            /// which means this function cannot ensure it always returns a valid reference:
            ///
            /// ```rust,compile_fail
            /// fn select<'a, 'b>(s1: &'a str, s2: &'b str, second: bool) -> &'a str
            /// {
            ///     if second { s2 } else { s1 }
            /// }
            /// ```
            ///
            /// `where` can also be used to express more complicated constraints that cannot
            /// be written with the `<T: Trait>` syntax:
            ///
            /// ```rust
            /// fn first_or_default<I>(mut i: I) -> I::Item
            /// where
            ///     I: Iterator,
            ///     I::Item: Default,
            /// {
            ///     i.next().unwrap_or_else(I::Item::default)
            /// }
            ///
            /// assert_eq!(first_or_default([1, 2, 3].into_iter()), 1);
            /// assert_eq!(first_or_default(Vec::<i32>::new().into_iter()), 0);
            /// ```
            ///
            /// `where` is available anywhere generic and lifetime parameters are available,
            /// as can be seen with the [`Cow`](crate::borrow::Cow) type from the standard
            /// library:
            ///
            /// ```rust
            /// # #![allow(dead_code)]
            /// pub enum Cow<'a, B>
            /// where
            ///     B: 'a + ToOwned + ?Sized,
            /// {
            ///     Borrowed(&'a B),
            ///     Owned(<B as ToOwned>::Owned),
            /// }
            /// ```
            ///
            /// [RFC]: https://github.com/rust-lang/rfcs/blob/master/text/0135-where.md
            mod where_keyword {}
        };

        let ref_keyword: ItemMod = parse_quote! {
            /// Bind by reference during pattern matching.
            ///
            /// `ref` annotates pattern bindings to make them borrow rather than move.
            /// It is **not** a part of the pattern as far as matching is concerned: it does
            /// not affect *whether* a value is matched, only *how* it is matched.
            ///
            /// By default, [`match`] statements consume all they can, which can sometimes
            /// be a problem, when you don't really need the value to be moved and owned:
            ///
            /// ```compile_fail,E0382
            /// let maybe_name = Some(String::from("Alice"));
            /// // The variable 'maybe_name' is consumed here ...
            /// match maybe_name {
            ///     Some(n) => println!("Hello, {n}"),
            ///     _ => println!("Hello, world"),
            /// }
            /// // ... and is now unavailable.
            /// println!("Hello again, {}", maybe_name.unwrap_or("world".into()));
            /// ```
            ///
            /// Using the `ref` keyword, the value is only borrowed, not moved, making it
            /// available for use after the [`match`] statement:
            ///
            /// ```
            /// let maybe_name = Some(String::from("Alice"));
            /// // Using `ref`, the value is borrowed, not moved ...
            /// match maybe_name {
            ///     Some(ref n) => println!("Hello, {n}"),
            ///     _ => println!("Hello, world"),
            /// }
            /// // ... so it's available here!
            /// println!("Hello again, {}", maybe_name.unwrap_or("world".into()));
            /// ```
            ///
            /// # `&` vs `ref`
            ///
            /// - `&` denotes that your pattern expects a reference to an object. Hence `&`
            /// is a part of said pattern: `&Foo` matches different objects than `Foo` does.
            ///
            /// - `ref` indicates that you want a reference to an unpacked value. It is not
            /// matched against: `Foo(ref foo)` matches the same objects as `Foo(foo)`.
            ///
            /// See also the [Reference] for more information.
            ///
            /// [`match`]: keyword.match.html
            /// [Reference]: ../reference/patterns.html#identifier-patterns
            mod ref_keyword {}
        };

        let true_keyword: ItemMod = parse_quote! {
            /// A value of type [`bool`] representing logical **true**.
            ///
            /// Logically `true` is not equal to [`false`].
            ///
            /// ## Control structures that check for **true**
            ///
            /// Several of Sway's control structures will check for a `bool` condition evaluating to **true**.
            ///
            ///   * The condition in an [`if`] expression must be of type `bool`.
            ///     Whenever that condition evaluates to **true**, the `if` expression takes
            ///     on the value of the first block. If however, the condition evaluates
            ///     to `false`, the expression takes on value of the `else` block if there is one.
            ///
            ///   * [`while`] is another control flow construct expecting a `bool`-typed condition.
            ///     As long as the condition evaluates to **true**, the `while` loop will continually
            ///     evaluate its associated block.
            ///
            ///   * [`match`] arms can have guard clauses on them.
            mod true_keyword {}
        };

        let false_keyword: ItemMod = parse_quote! {
            /// A value of type [`bool`] representing logical **false**.
            ///
            /// `false` is the logical opposite of [`true`].
            ///
            /// See the documentation for [`true`] for more information.
            mod false_keyword {}
        };

        let break_keyword: ItemMod = parse_quote! {
            /// Exit early from a loop.
            ///
            /// When `break` is encountered, execution of the associated loop body is
            /// immediately terminated.
            ///
            /// ```rust
            /// let mut last = 0;
            ///
            /// for x in 1..100 {
            ///     if x > 12 {
            ///         break;
            ///     }
            ///     last = x;
            /// }
            ///
            /// assert_eq!(last, 12);
            /// println!("{last}");
            /// ```
            ///
            /// A break expression is normally associated with the innermost loop enclosing the
            /// `break` but a label can be used to specify which enclosing loop is affected.
            ///
            /// ```rust
            /// 'outer: for i in 1..=5 {
            ///     println!("outer iteration (i): {i}");
            ///
            ///     '_inner: for j in 1..=200 {
            ///         println!("    inner iteration (j): {j}");
            ///         if j >= 3 {
            ///             // breaks from inner loop, lets outer loop continue.
            ///             break;
            ///         }
            ///         if i >= 2 {
            ///             // breaks from outer loop, and directly to "Bye".
            ///             break 'outer;
            ///         }
            ///     }
            /// }
            /// println!("Bye.");
            /// ```
            ///
            /// When associated with `loop`, a break expression may be used to return a value from that loop.
            /// This is only valid with `loop` and not with any other type of loop.
            /// If no value is specified, `break;` returns `()`.
            /// Every `break` within a loop must return the same type.
            ///
            /// ```rust
            /// let (mut a, mut b) = (1, 1);
            /// let result = loop {
            ///     if b > 10 {
            ///         break b;
            ///     }
            ///     let c = a + b;
            ///     a = b;
            ///     b = c;
            /// };
            /// // first number in Fibonacci sequence over 10:
            /// assert_eq!(result, 13);
            /// println!("{result}");
            /// ```
            ///
            /// For more details consult the [Reference on "break expression"] and the [Reference on "break and
            /// loop values"].
            ///
            /// [Reference on "break expression"]: ../reference/expressions/loop-expr.html#break-expressions
            /// [Reference on "break and loop values"]:
            /// ../reference/expressions/loop-expr.html#break-and-loop-values
            mod break_keyword {}
        };

        let continue_keyword: ItemMod = parse_quote! {
            /// Skip to the next iteration of a loop.
            ///
            /// When `continue` is encountered, the current iteration is terminated, returning control to the
            /// loop head, typically continuing with the next iteration.
            ///
            /// ```rust
            /// // Printing odd numbers by skipping even ones
            /// for number in 1..=10 {
            ///     if number % 2 == 0 {
            ///         continue;
            ///     }
            ///     println!("{number}");
            /// }
            /// ```
            ///
            /// Like `break`, `continue` is normally associated with the innermost enclosing loop, but labels
            /// may be used to specify the affected loop.
            ///
            /// ```rust
            /// // Print Odd numbers under 30 with unit <= 5
            /// 'tens: for ten in 0..3 {
            ///     '_units: for unit in 0..=9 {
            ///         if unit % 2 == 0 {
            ///             continue;
            ///         }
            ///         if unit > 5 {
            ///             continue 'tens;
            ///         }
            ///         println!("{}", ten * 10 + unit);
            ///     }
            /// }
            /// ```
            ///
            /// See [continue expressions] from the reference for more details.
            ///
            /// [continue expressions]: ../reference/expressions/loop-expr.html#continue-expressions
            mod continue_keyword {}
        };

        // SWAY SPECIFIC
        let script_keyword: ItemMod = parse_quote! {
            /// TODO
            mod script_keyword {}
        };
        let contract_keyword: ItemMod = parse_quote! {
            /// TODO
            mod contract_keyword {}
        };
        let predicate_keyword: ItemMod = parse_quote! {
            /// TODO
            mod predicate_keyword {}
        };
        let library_keyword: ItemMod = parse_quote! {
            /// TODO
            mod library_keyword {}
        };
        let dep_keyword: ItemMod = parse_quote! {
            /// TODO
            mod dep_keyword {}
        };
        let abi_keyword: ItemMod = parse_quote! {
            /// TODO
            mod abi_keyword {}
        };
        let storage_keyword: ItemMod = parse_quote! {
            /// TODO
            mod storage_keyword {}
        };
        let asm_keyword: ItemMod = parse_quote! {
            /// TODO
            mod asm_keyword {}
        };
        let deref_keyword: ItemMod = parse_quote! {
            /// TODO
            mod deref_keyword {}
        };
        let configurable_keyword: ItemMod = parse_quote! {
            /// TODO
            mod configurable_keyword {}
        };

        let mut keyword_docs = HashMap::new();

        let keywords = vec![
            pub_keyword,
            use_keyword,
            as_keyword,
            struct_keyword,
            enum_keyword,
            self_keyword,
            fn_keyword,
            trait_keyword,
            impl_keyword,
            for_keyword,
            const_keyword,
            return_keyword,
            if_keyword,
            else_keyword,
            match_keyword,
            mut_keyword,
            let_keyword,
            while_keyword,
            where_keyword,
            ref_keyword,
            true_keyword,
            false_keyword,
            break_keyword,
            continue_keyword,
            script_keyword,
            contract_keyword,
            predicate_keyword,
            library_keyword,
            dep_keyword,
            abi_keyword,
            storage_keyword,
            asm_keyword,
            deref_keyword,
            configurable_keyword,
        ];

        keywords.iter().for_each(|keyword| {
            let ident = keyword.ident.clone().to_string();
            // remove "_keyword" suffix to get the keyword name
            let name = ident.trim_end_matches("_keyword").to_owned();
            let mut documentation = String::new();
            keyword.attrs.iter().for_each(|attr| {
                let tokens = attr.tokens.to_token_stream();
                let lit = extract_lit(tokens);
                writeln!(documentation, "{}", lit).unwrap();
            });
            keyword_docs.insert(name, documentation);
        });

        Self {
            inner: keyword_docs,
        }
    }

    /// Returns the documentation for the given keyword.
    pub fn get(&self, keyword: &str) -> Option<&String> {
        self.inner.get(keyword)
    }
}

#[test]
fn test2() {
    let kw_docs = KeywordDocs::new();
    eprintln!("{}", kw_docs.get("true").unwrap());
}

/// Extracts the literal from a token stream and returns it as a string.
fn extract_lit(tokens: TokenStream) -> String {
    let mut res = "".to_string();
    for token in tokens.into_iter() {
        if let TokenTree::Literal(l) = token {
            let mut s = l.to_string();
            s = s.replace("r\"", "///"); // replace the "r\"" with /// at the beginning
            s.pop(); // remove the " at the end
            res.push_str(&s);
        }
    }
    res
}
