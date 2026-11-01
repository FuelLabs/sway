script;

// Tests that the provided methods of a trait remain correctly bound to the trait
// and to each of the implementing types.
//
// Type checking an `impl` of a trait derives the impl's own versions of the
// trait's provided methods. This test proves that deriving those versions while
// type checking later impls does not corrupt the provided methods of the
// previously type-checked impls, or of the trait declaration itself.
//
// Therefore, all the calls in `main` are done after all the impls are type
// checked, and the provided methods are called both directly on the concrete
// types and through trait constraints, where the methods are resolved through
// the trait declaration's items.

trait Describe {
    fn value(self) -> u64;
} {
    // A provided method calling an interface method, inherited by all the impls.
    fn describe(self) -> u64 {
        self.value() * 10
    }
    // Another provided method calling an interface method.
    fn describe_scaled(self) -> u64 {
        self.value() * 100
    }
    // A provided associated function that mentions neither `Self` nor any
    // generic parameter.
    fn describe_tag() -> u64 {
        7
    }
}

// A generic trait, whose provided method gets monomorphized for every impl.
trait Pair<T> {
    fn first(self) -> u64;
    fn second(self) -> T;
} {
    // A provided method that does not mention `T` in its body.
    fn first_doubled(self) -> u64 {
        self.first() * 2
    }
}

// A trait whose methods and associated functions are called in the provided
// methods of the generic `Lookup` trait below.
trait Key {
    fn key(self) -> u64;
    fn default_key() -> u64;
}

// A generic trait with a constraint on its type parameter. Its provided methods
// and associated function call methods and associated functions of the `Key`
// trait on `T`.
trait Lookup<T>
where
    T: Key,
{
    fn base(self) -> u64;
} {
    // A generic provided method calling a method of `Key` on a `T` instance.
    fn key_of(self, t: T) -> u64 {
        self.base() + t.key()
    }
    // A generic provided method calling an associated function of `Key`.
    fn base_plus_default_key(self) -> u64 {
        self.base() + T::default_key()
    }
    // A generic provided associated function calling an associated function
    // of `Key`.
    fn double_default_key() -> u64 {
        T::default_key() * 2
    }
}

struct A {}
struct B {}
struct C {}

impl Describe for A {
    fn value(self) -> u64 {
        1
    }
}

impl Describe for B {
    fn value(self) -> u64 {
        2
    }
}

impl Describe for C {
    fn value(self) -> u64 {
        3
    }
}

impl Pair<u64> for A {
    fn first(self) -> u64 {
        10
    }
    fn second(self) -> u64 {
        42
    }
}

impl Pair<bool> for B {
    fn first(self) -> u64 {
        20
    }
    fn second(self) -> bool {
        true
    }
}

impl Pair<u64> for C {
    fn first(self) -> u64 {
        30
    }
    fn second(self) -> u64 {
        43
    }
}

impl Key for A {
    fn key(self) -> u64 {
        5
    }
    fn default_key() -> u64 {
        50
    }
}

impl Key for B {
    fn key(self) -> u64 {
        6
    }
    fn default_key() -> u64 {
        60
    }
}

impl Lookup<A> for B {
    fn base(self) -> u64 {
        1000
    }
}

impl Lookup<B> for C {
    fn base(self) -> u64 {
        2000
    }
}

// The provided methods called here are resolved through the trait declaration's
// items, based on the trait constraint.
fn describe_of<T>(t: T) -> u64
where
    T: Describe,
{
    t.describe()
}

fn describe_scaled_of<T>(t: T) -> u64
where
    T: Describe,
{
    t.describe_scaled()
}

fn describe_tag_of<T>() -> u64
where
    T: Describe,
{
    T::describe_tag()
}

fn first_doubled_of<T>(t: T) -> u64
where
    T: Pair<u64>,
{
    t.first_doubled()
}

fn key_of_through_constraint<T, U>(t: T, u: U) -> u64
where
    U: Key,
    T: Lookup<U>,
{
    t.key_of(u)
}

fn double_default_key_of<T, U>() -> u64
where
    U: Key,
    T: Lookup<U>,
{
    T::double_default_key()
}

fn main() -> u64 {
    let a = A {};
    let b = B {};
    let c = C {};

    // Direct calls on the concrete types.
    assert(a.describe() == 10);
    assert(b.describe() == 20);
    assert(c.describe() == 30);

    assert(a.describe_scaled() == 100);
    assert(b.describe_scaled() == 200);
    assert(c.describe_scaled() == 300);

    assert(A::describe_tag() == 7);
    assert(B::describe_tag() == 7);
    assert(C::describe_tag() == 7);

    assert(a.first_doubled() == 20);
    assert(b.first_doubled() == 40);
    assert(c.first_doubled() == 60);

    // Calls through trait constraints.
    assert(describe_of(a) == 10);
    assert(describe_of(b) == 20);
    assert(describe_of(c) == 30);

    assert(describe_scaled_of(a) == 100);
    assert(describe_scaled_of(b) == 200);
    assert(describe_scaled_of(c) == 300);

    assert(describe_tag_of::<A>() == 7);
    assert(describe_tag_of::<B>() == 7);
    assert(describe_tag_of::<C>() == 7);

    assert(first_doubled_of(a) == 20);
    assert(first_doubled_of(c) == 60);

    // Direct calls of the generic provided methods that call methods and
    // associated functions of the `Key` trait.
    assert(b.key_of(a) == 1005);
    assert(c.key_of(b) == 2006);

    assert(b.base_plus_default_key() == 1050);
    assert(c.base_plus_default_key() == 2060);

    assert(B::double_default_key() == 100);
    assert(C::double_default_key() == 120);

    // The same, but through trait constraints.
    assert(key_of_through_constraint(b, a) == 1005);
    assert(key_of_through_constraint(c, b) == 2006);

    assert(double_default_key_of::<B, A>() == 100);
    assert(double_default_key_of::<C, B>() == 120);

    // Fully qualified calls of the provided methods.
    assert(<A as Describe>::describe(a) == 10);
    assert(<B as Describe>::describe_scaled(b) == 200);
    assert(<B as Lookup<A>>::double_default_key() == 100);

    42
}
