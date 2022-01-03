# Reference Types

If you have familiarity with references, also called pointers, from other languages, Sway references are no different. If you're new to this concept, this chapter is for you!

Memory in a computer is held in RAM. When you buy a Macbook Pro 16GB, for example, that 16GB number is referring to the _memory_, or _RAM_, available. In the FuelVM, we also have memory. When you instantiate a variable in a Sway program, it is written to some spot in memory. We need to keep track of _where_ exactly that value was written in order to utilize it, though.

Every single byte in FuelVM memory has a name. The first byte's name is `0x01`. The second byte's name is `0x02`. The 54,321st byte's name is `0xD431`[^1]. A reference is a variable which contains the name of a specific location in the FuelVM's memory. This is useful if you want to reason about the memory which contains the value and not the value itself.

```sway
script;
fn main() {
    let x = 42;
    let reference_to_x = ref x;
}
```

[^1]: Check out [this article](https://en.wikipedia.org/wiki/Hexadecimal) if you're not used to seeing numbers with letters in them. 

## Dereferencing

## Storage References

## Implicit References

Types larger than one word in size are implicitly reference types.