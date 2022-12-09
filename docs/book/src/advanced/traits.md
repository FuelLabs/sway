# Traits

## Declaring a Trait

A _trait_ opts a type into a certain type of behavior or functionality that can be shared among types. This allows for easy reuse of code and generic programming. If you have ever used a typeclass in Haskell, a trait in Rust, or even an interface in Java, these are similar concepts.

Let's take a look at some code:

```sway
trait Compare {
    fn equals(self, b: Self) -> bool;
} {
    fn not_equals(self, b: Self) -> bool {
        !self.equals(b)
    }
}
```

We have just declared a trait called `Compare`. After the name of the trait, there are two _blocks_ of code (a _block_ is code enclosed in `{` curly brackets `}`). The first block is the _interface surface_. The second block is the _methods_ provided by the trait. If a type can provide the methods in the interface surface, then it gets access to the methods in the trait for free! What the above trait is saying is: if you can determine if two values are equal, then for free, you can determine that they are not equal. Note that trait methods have access to the methods defined in the interface surface.

## Implementing a Trait

Ok, so I know that numbers can be equal. I want to implement my `Compare` trait for `u64`. Let's take a look at how that is done:

```sway
impl Compare for u64 {
    fn equals(self, b: Self) -> bool {
        self == b
    }
}
```

The above snippet declares all of the methods in the trait `Compare` for the type `u64`. Now, we have access to both the `equals` and `not_equals` methods for `u64`, as long as the trait `Compare` is in scope.

## Supertraits

When using multiple traits, scenarios often come up where one trait may require functionality from another trait. This is where supertraits come in as they allow you to require a trait when implementing another
trait (ie. a trait with a trait). A good example of this is the `Ord` trait of the `core` library of Sway. The `Ord` trait requires the `Eq` trait, so `Eq` is kept as a separate trait as one may decide to implement `Eq`
without implementing other parts of the `Ord` trait.

```sway

trait Eq {
    fn equals(self, b: Self) -> bool;
}

trait Ord: Eq {
    fn gte(self, b: Self) -> bool;
}

impl Ord for u64 {
    fn gte(self, b: Self) -> bool {
        // As `Eq` is a supertrait of `Ord`, `Ord` can access the equals method
        self.equals(b) || self.gt(b)
    }
}
```

To require a supertrait, add a `:` after the trait name and then list the traits you would like to require and separate them with a `+`.

## Use Cases

### Custom Types (structs, enums)

Often, libraries and APIs have interfaces that are abstracted over a type that implements a certain trait. It is up to the consumer of the interface to implement that trait for the type they wish to use with the interface. For example, let's take a look at a trait and an interface built off of it.

```sway
library games;

pub enum Suit {
    Hearts: (),
    Diamonds: (),
    Clubs: (),
    Spades: (),
}

pub trait Card {
    fn suit(self) -> Suit;
    fn value(self) -> u8;
}

fn play_game_with_deck<T>(a: Vec<T>) where T: Card {
    // insert some creative card game here
}
```

> **Note** Trait constraints (i.e. using the `where` keyword) [have not yet been implemented](https://github.com/FuelLabs/sway/issues/970)

Now, if you want to use the function `play_game_with_deck` with your struct, you must implement `Card` for your struct. Note that the following code example assumes a dependency _games_ has been included in the `Forc.toml` file.

```sway
script;

use games::*;

struct MyCard {
    suit: Suit,
    value: u8
}

impl Card for MyCard {
    fn suit(self) -> Suit {
        self.suit
    }
    fn value(self) -> u8 {
        self.value
    }
}

fn main() {
    let mut i = 52;
    let mut deck: Vec<MyCard> = Vec::with_capacity(50);
    while i > 0 {
        i = i - 1;
        deck.push(MyCard { suit: generate_random_suit(), value: i % 4}
    }
    play_game_with_deck(deck);
}

fn generate_random_suit() -> Suit {
  [ ... ]
}
```
