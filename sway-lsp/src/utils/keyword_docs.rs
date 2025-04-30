#![allow(unused)]

use proc_macro2::{TokenStream, TokenTree};
use quote::ToTokens;
use std::{collections::hash_map::HashMap, fmt::Write};
use syn::{parse_quote, ItemMod};

// Ported from https://github.com/rust-lang/rust/blob/master/library/std/src/keyword_docs.rs

/// Documentation for sway keywords.
/// Primarily used for showing documentation on LSP hover requests.
/// Key = keyword
/// Value = documentation
#[derive(Debug, Default)]
pub struct KeywordDocs(HashMap<String, String>);

impl KeywordDocs {
    pub fn new() -> Self {
        let pub_keyword: ItemMod = parse_quote! {
            /// Make an item visible to others.
            ///
            /// The keyword `pub` makes any module, function, or data structure accessible from inside
            /// of external modules. The `pub` keyword may also be used in a `use` declaration to re-export
            /// an identifier from a namespace.
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
            mod use_keyword {}
        };

        let as_keyword: ItemMod = parse_quote! {
            /// Cast between types, or rename an import.
            ///
            /// In general, any cast that can be performed via ascribing the type can also be done using `as`,
            /// so instead of writing `let x: u32 = 123`, you can write `let x = 123 as u32` (note: `let x: u32
            /// = 123` would be best in that situation). The same is not true in the other direction
            ///
            /// `as` can also be used with the `_` placeholder when the destination type can be inferred. Note
            /// that this can cause inference breakage and usually such code should use an explicit type for
            /// both clarity and stability.
            ///
            /// `as` is also used to rename imports in [`use`] statements:
            ///
            /// ```sway
            /// use foo::Foo as MyFoo;
            /// ```
            mod as_keyword {}
        };

        let struct_keyword: ItemMod = parse_quote! {
            /// A type that is composed of other types.
            ///
            /// Structs in Sway come in three flavors: Structs with named fields, tuple structs, and unit
            /// structs.
            ///
            /// ```sway
            /// struct Regular {
            ///     field1: u8,
            ///     field2: u32,
            ///     pub field3: bool
            /// }
            ///
            /// struct Tuple(u32, u64);
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
            /// ```sway
            /// # struct Foo { field1: u8, field2: u32, etc: bool }
            /// let example = Foo {
            ///     field1: 42,
            ///     field2: 1024,
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
            /// ```sway
            /// struct User {
            ///     age: u8,
            ///     admin: bool,
            /// }
            ///
            /// impl User {
            ///     pub fn new(age: u8) -> Self {
            ///         Self {
            ///             age,
            ///             admin: false,
            ///         }
            ///     }
            /// }
            /// ```
            ///
            /// Tuple structs are instantiated in the same way as tuples themselves, except with the struct's
            /// name as a prefix: `Foo(123, false, 26)`.
            ///
            /// Empty structs are instantiated with just their name, and don't need anything else. `let thing =
            /// EmptyStruct;`
            ///
            /// # Style conventions
            ///
            /// Structs are always written in CamelCase, with few exceptions. While the trailing comma on a
            /// struct's list of fields can be omitted, it's usually kept for convenience in adding and
            /// removing fields down the line.
            mod struct_keyword {}
        };

        let enum_keyword: ItemMod = parse_quote! {
            /// A type that can be any one of several variants.
            ///
            /// Enums in Sway are similar to those of other compiled languages like C, but have important
            /// differences that make them considerably more powerful. What Sway calls enums are more commonly
            /// known as [Algebraic Data Types][ADT] if you're coming from a functional programming background.
            /// The important detail is that each enum variant can have data to go along with it.
            ///
            /// ```sway
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
            ///         blah: u8,
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
            /// When data follows along with a variant, such as with sway's built-in [`Option`] type, the data
            /// is added as the type describes, for example `Option::Some(123)`. The same follows with
            /// struct-like variants, with things looking like `ComplexEnum::LotsOfThings { usual_struct_stuff:
            /// true, blah: 245, }`. Empty Enums are similar to [`!`] in that they cannot be
            /// instantiated at all, and are used mainly to mess with the type system in interesting ways.
            ///
            /// [ADT]: https://en.wikipedia.org/wiki/Algebraic_data_type
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
            /// ```sway
            /// use std::contract_id::{self, ContractId};
            /// ```
            ///
            /// Is functionally the same as:
            ///
            /// ```sway
            /// use std::contract_id;
            /// use std::contract_id::ContractId;
            /// ```
            ///
            /// `self` as the current receiver for a method allows to omit the parameter
            /// type most of the time. With the exception of this particularity, `self` is
            /// used much like any other parameter:
            ///
            /// ```sway
            /// struct Foo(u32);
            ///
            /// impl Foo {
            ///     // No `self`.
            ///     fn new() -> Self {
            ///         Self(0)
            ///     }
            ///
            ///     // Borrowing `self`.
            ///     fn value(&self) -> u32 {
            ///         self.0
            ///     }
            ///
            ///     // Updating `self` mutably.
            ///     fn clear(ref mut self) {
            ///         self.0 = 0
            ///     }
            /// }
            /// ```
            mod self_keyword {}
        };

        let fn_keyword: ItemMod = parse_quote! {
            /// Functions are the primary way code is executed within Sway. Function blocks, usually just
            /// called functions, can be defined in a variety of different places and be assigned many
            /// different attributes and modifiers.
            ///
            /// Standalone functions that just sit within a module not attached to anything else are common,
            /// but most functions will end up being inside [`impl`] blocks, either on another type itself, or
            /// as a trait impl for that type.
            ///
            /// ```sway
            /// fn standalone_function() {
            ///     // code
            /// }
            ///
            /// pub fn public_thing(argument: bool) -> bool {
            ///     // code
            ///     true
            /// }
            ///
            /// struct Thing {
            ///     foo: u32,
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
            /// ```sway
            /// fn add_points<T>(a: MyPoint<T>, b: MyPoint<T>) -> MyPoint<T> where T: MyAdd {
            ///    MyPoint {
            ///        x: a.x.my_add(b.x),
            ///        y: a.y.my_add(b.y),
            ///    }
            /// }
            /// ```
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
            /// trait it is promising to uphold its contract.
            ///
            /// # Examples
            ///
            /// Traits are declared using the `trait` keyword. Types can implement them
            /// using [`impl`] `Trait` [`for`] `Type`:
            ///
            /// ```sway
            /// trait Setter<T> {
            ///     fn set(self, new_value: T) -> Self;
            /// }
            ///
            /// struct FooBarData<T> {
            ///     value: T
            /// }
            ///
            /// impl<T> Setter<T> for FooBarData<T> {
            ///     fn set(self, new_value: T) -> Self {
            ///         FooBarData {
            ///             value: new_value,
            ///         }
            ///     }
            /// }
            /// ```
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
            /// takes `self`, or `ref mut self` as its first argument, it can also be called using
            /// method-call syntax, a familiar feature to any object oriented programmer, like `foo.bar()`.
            ///
            /// ```sway
            /// struct Example {
            ///     number: u32,
            /// }
            ///
            /// impl Example {
            ///     fn answer(ref mut self) {
            ///         self.number += 42;
            ///     }
            ///
            ///     fn get_number(self) -> u32 {
            ///         self.number
            ///     }
            /// }
            /// ```
            mod impl_keyword {}
        };

        let const_keyword: ItemMod = parse_quote! {
            /// Compile-time constants.
            ///
            /// Sometimes a certain value is used many times throughout a program, and it can become
            /// inconvenient to copy it over and over. What's more, it's not always possible or desirable to
            /// make it a variable that gets carried around to each function that needs it. In these cases, the
            /// `const` keyword provides a convenient alternative to code duplication:
            ///
            /// ```sway
            /// const NUMBER_1: u64 = 7;
            ///
            /// let foo = 123 + NUMBER_1;
            /// ```
            ///
            /// Constants must be explicitly typed; unlike with `let`, you can't ignore their type and let the
            /// compiler figure it out.
            ///
            /// Constants should always be in `SCREAMING_SNAKE_CASE`.
            mod const_keyword {}
        };

        let return_keyword: ItemMod = parse_quote! {
            /// Return a value from a function.
            ///
            /// A `return` marks the end of an execution path in a function:
            ///
            /// ```sway
            /// fn foo() -> u32 {
            ///     return 3;
            /// }
            /// assert(foo(), 3);
            /// ```
            ///
            /// `return` is not needed when the returned value is the last expression in the
            /// function. In this case the `;` is omitted:
            ///
            /// ```sway
            /// fn foo() -> u32 {
            ///     3
            /// }
            /// assert(foo(), 3);
            /// ```
            ///
            /// `return` returns from the function immediately (an "early return"):
            ///
            /// ```sway
            /// fn main() -> u64 {
            ///     let x = if true {
            ///         Result::Err::<u64, u32>(12)
            ///     } else {
            ///         return 10;
            ///     };
            ///     44
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
            /// ```sway
            /// if 1 == 2 {
            ///     log("whoops, mathematics broke");
            ///     revert(0);
            /// } else {
            ///     log("everything's fine!");
            /// }
            ///
            /// let x = 5;
            /// let y = if x == 5 {
            ///     10
            /// } else {
            ///     15
            /// };
            /// assert(y == 10);
            ///
            /// let opt = Some(5);
            /// if let Some(x) = opt {
            ///    // x is 5
            /// }
            /// ```
            ///
            /// Shown above are the three typical forms an `if` block comes in. First is the usual kind of
            /// thing you'd see in many languages, with an optional `else` block. Second uses `if` as an
            /// expression, which is only possible if all branches return the same type. An `if` expression can
            /// be used everywhere you'd expect. The third kind of `if` block is an `if let` block, which
            /// behaves similarly to using a `match` expression.
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
            /// ```sway
            /// let condition = false;
            /// let result = if condition == true {
            ///     101
            /// } else {
            ///     102
            /// };
            /// assert(result == 102);
            /// ```
            ///
            /// There is possibly no limit to the number of `else` blocks that could follow an `if` expression
            /// however if you have several then a [`match`] expression might be preferable.
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
            /// ```sway
            /// let opt = None::<u32>;
            /// let x = match opt {
            ///     Some(int) => int,
            ///     None => 10,
            /// };
            /// assert(x, 10);
            /// ```
            mod match_keyword {}
        };

        let mut_keyword: ItemMod = parse_quote! {
            /// A mutable variable, reference, or pointer.
            ///
            /// `mut` can be used in several situations. The first is mutable variables,
            /// which can be used anywhere you can bind a value to a variable name. Some
            /// examples:
            ///
            /// ```sway
            /// let mut a = 5;
            /// a = 6;
            /// assert(a, 6);
            /// ```
            ///
            /// The second is mutable references. They can be created from `mut` variables
            /// and must be unique: no other variables can have a mutable reference, nor a
            /// shared reference.
            ///
            /// ```sway
            /// // Taking a mutable reference.
            /// fn takes_ref_mut_array(ref mut arr: [u64; 1]) {
            ///     arr[0] = 10;
            /// }
            /// ```
            mod mut_keyword {}
        };

        let let_keyword: ItemMod = parse_quote! {
            /// Bind a value to a variable.
            ///
            /// The primary use for the `let` keyword is in `let` statements, which are used to introduce a new
            /// set of variables into the current scope, as given by a pattern.
            ///
            /// ```sway
            /// let thing1: u32 = 100;
            /// let thing2 = 200 + thing1;
            ///
            /// let mut changing_thing = true;
            /// changing_thing = false;
            /// ```
            ///
            /// The pattern is most commonly a single variable, which means no pattern matching is done and
            /// the expression given is bound to the variable. Apart from that, patterns used in `let` bindings
            /// can be as complicated as needed, given that the pattern is exhaustive. The type of the pattern
            /// is optionally given afterwards, but if left blank is automatically inferred by the compiler if possible.
            ///
            /// Variables in Sway are immutable by default, and require the `mut` keyword to be made mutable.
            ///
            /// Multiple variables can be defined with the same name, known as shadowing. This doesn't affect
            /// the original variable in any way beyond being unable to directly access it beyond the point of
            /// shadowing. It continues to remain in scope, getting dropped only when it falls out of scope.
            /// Shadowed variables don't need to have the same type as the variables shadowing them.
            ///
            /// ```sway
            /// let shadowing_example = true;
            /// let shadowing_example: u32 = 123;
            /// let shadowing_example = shadowing_example as u8;
            /// ```
            ///
            /// Other places the `let` keyword is used include along with [`if`], in the form of `if let`
            /// expressions. They're useful if the pattern being matched isn't exhaustive, such as with
            /// enumerations.
            mod let_keyword {}
        };

        let while_keyword: ItemMod = parse_quote! {
            /// Loop while a condition is upheld.
            ///
            /// A `while` expression is used for predicate loops. The `while` expression runs the conditional
            /// expression before running the loop body, then runs the loop body if the conditional
            /// expression evaluates to `true`, or exits the loop otherwise.
            ///
            /// ```sway
            /// let mut counter = 0;
            ///
            /// while counter < 10 {
            ///     log(counter);
            ///     counter += 1;
            /// }
            /// ```
            ///
            /// A `while` expression cannot break with a value and always evaluates to `()`.
            ///
            /// ```sway
            /// let mut i = 1;
            ///
            /// while i < 100 {
            ///     i *= 2;
            ///     if i == 64 {
            ///         break; // Exit when `i` is 64.
            ///     }
            /// }
            /// ```
            mod while_keyword {}
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
            /// ```sway
            /// let mut x = 0;
            ///
            /// for x < 100 {
            ///     if x > 12 {
            ///         break;
            ///     }
            ///     x += 1;
            /// }
            ///
            /// assert(x == 12);
            /// ```
            mod break_keyword {}
        };

        let continue_keyword: ItemMod = parse_quote! {
            /// Skip to the next iteration of a loop.
            ///
            /// When `continue` is encountered, the current iteration is terminated, returning control to the
            /// loop head, typically continuing with the next iteration.
            ///
            /// ```sway
            /// // Printing odd numbers by skipping even ones
            /// for number in 1..=10 {
            ///     if number % 2 == 0 {
            ///         continue;
            ///     }
            ///     log(number);
            /// }
            /// ```
            mod continue_keyword {}
        };

        // TODO
        let str_keyword: ItemMod = parse_quote! {
            mod str_keyword {}
        };

        // TODO
        let for_keyword: ItemMod = parse_quote! {
            mod for_keyword {}
        };

        // TODO
        let where_keyword: ItemMod = parse_quote! {
            mod where_keyword {}
        };

        // TODO
        let ref_keyword: ItemMod = parse_quote! {
            mod ref_keyword {}
        };

        // TODO
        let script_keyword: ItemMod = parse_quote! {
            mod script_keyword {}
        };

        // TODO
        let contract_keyword: ItemMod = parse_quote! {
            mod contract_keyword {}
        };

        // TODO
        let predicate_keyword: ItemMod = parse_quote! {
            mod predicate_keyword {}
        };

        // TODO
        let library_keyword: ItemMod = parse_quote! {
            mod library_keyword {}
        };

        // TODO
        let mod_keyword: ItemMod = parse_quote! {
            mod mod_keyword {}
        };

        // TODO
        let abi_keyword: ItemMod = parse_quote! {
            mod abi_keyword {}
        };

        // TODO
        let storage_keyword: ItemMod = parse_quote! {
            mod storage_keyword {}
        };

        // TODO
        let asm_keyword: ItemMod = parse_quote! {
            mod asm_keyword {}
        };

        // TODO
        let deref_keyword: ItemMod = parse_quote! {
            mod deref_keyword {}
        };

        // TODO
        let configurable_keyword: ItemMod = parse_quote! {
            mod configurable_keyword {}
        };

        // TODO
        let type_keyword: ItemMod = parse_quote! {
            mod type_keyword {}
        };

        // TODO
        let panic_keyword: ItemMod = parse_quote! {
            mod panic_keyword {}
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
            str_keyword,
            script_keyword,
            contract_keyword,
            predicate_keyword,
            library_keyword,
            mod_keyword,
            abi_keyword,
            storage_keyword,
            asm_keyword,
            deref_keyword,
            configurable_keyword,
            type_keyword,
            panic_keyword,
        ];

        for keyword in &keywords {
            let ident = keyword.ident.clone().to_string();
            // remove "_keyword" suffix to get the keyword name
            let name = ident.trim_end_matches("_keyword").to_owned();
            let mut documentation = String::new();
            keyword.attrs.iter().for_each(|attr| {
                let tokens = attr.meta.clone().to_token_stream();
                let lit = extract_lit(tokens);
                writeln!(documentation, "{lit}").unwrap();
            });
            keyword_docs.insert(
                name,
                documentation.replace("///\n", "\n").replace("/// ", ""),
            );
        }

        Self(keyword_docs)
    }
}

impl std::ops::Deref for KeywordDocs {
    type Target = HashMap<String, String>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Extracts the literal from a token stream and returns it as a string.
fn extract_lit(tokens: TokenStream) -> String {
    let mut res = String::new();
    for token in tokens {
        if let TokenTree::Literal(l) = token {
            let s = l.to_string();
            let s = s.trim_start_matches("r\""); // remove the r" sequence at the beginning
            let s = s.trim_end_matches('\"'); // remove the " at the end
            res.push_str(s);
        }
    }
    res
}

#[tokio::test]
async fn keywords_in_sync() {
    let keyword_docs = KeywordDocs::new();
    let lsp_keywords: Vec<_> = keyword_docs.keys().collect();
    let compiler_keywords: Vec<_> = sway_parse::RESERVED_KEYWORDS
        .iter()
        .map(|s| s.to_string())
        .collect();

    for keyword in &compiler_keywords {
        assert!(
            lsp_keywords.contains(&keyword),
            "Error: Documentation for the `{keyword}` keyword is not implemented in LSP"
        );
    }
}
