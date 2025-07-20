library;

struct S<T> { }

trait Trait { }

impl<A> Trait for &S<A> { }
impl<B> Trait for &S<B> { }

impl<A> Trait for &mut S<A> { }
impl<B> Trait for &mut S<B> { }

impl<A> Trait for & &S<A> { }
impl<B> Trait for & &S<B> { }

impl<A> Trait for &mut &mut S<A> { }
impl<B> Trait for &mut &mut S<B> { }

impl<A> Trait for & & &S<A> { }
impl<B> Trait for & & &S<B> { }

impl<A> Trait for &mut &mut &mut S<A> { }
impl<B> Trait for &mut &mut &mut S<B> { }

impl<A> Trait for &mut & &mut S<A> { }
impl<B> Trait for &mut & &mut S<B> { }