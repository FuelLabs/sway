# Struct Memory Layout

Structs have zero memory overhead meaning that each field is laid out sequentially in memory. No metadata regarding the struct's name or other properties is preserved at runtime. 

In other words, structs are compile-time constructs similar to Rust, but different in other languages with runtimes like Java.
