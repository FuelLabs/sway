script;

// A developer familiar with Rust might be tempted to define enum variants as `Foo` or `Bar(u32)`.
// This file tests the error message that informs the developer of the correct syntax.

enum Enum1 {
  Ok,    // Illegal
  Err,   // Also illegal, but shadowed by previous error
}

enum Enum2 {
  F(u32),        // Illegal
  G(u32, u32),   // Also illegal, but shadowed by previous error
}

fn main() {
}

