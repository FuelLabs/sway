# Trivially Encodable and Decodable

When a contract calls another contract all arguments are encoded just before the contract code is called, and it is decoded after the contract code is executing, but just before the contract method start executing.

This process is needed to increase the security of contract but also to facilitate development. Unfortunately it has a cost involved, that can const hundreads if not thousands of gas.

To alleviate this issue, the compiler completely bypass encoding and decoding for some types. For these types, the compiler can safely do this because their "runtime representation", how their bytes are allocated inside the VM, is exactly the same as their "encoding representation", how the bytes would be organized in the final buffer after encoding.

If both representations are guaranteed to always be the same, we say that they are "trivially encodable" or "trivially decodable".

Given the advantages all contract methods should be considered to have trivially encodable/decodable arguments. To achieve that the compiler has an attribute that fails the build when the specified type is no "trivially encodable/decodable". For example:

```sway
#[trivial(encode = "require", decode = "require")]
pub struct SomeArgument {
    ...
}
```

# Type that are never trivially encodable/decodable

Unfortunately some types, even base types are never trivial. The simplest example is `bool`. It is "trivially encodable", given that both representations are just one byte (0 or 1); but bool cannot be "trivially decodable", at least not safely trivially decodable. One just have to imagine that a simple buffer with one byte value "2"
cannot be trivially decoded into a bool, because the bool would have value "2" on runtime, and that is undefined behaviour.

Same thing happens with enums. They are prefixed with a `u64` specifying their variant. But given that enums will never fill all options, we cannot consider enums as "trivially decodable".

The "easy" and "obvious" solution is to not use `bool` and enums. Just use `u64` and check manually if the value is less than two. For enums? Do the same, use `u64` and check the range manually.

But this is far from ideal...

Another solution is to force these types to be trivial encodable/decodable. For `bool` and enums this can easily be done using `TriviaBool` and `TrivialEnum`. For example:

```sway
#[trivial(encode = "require", decode = "require")]
pub struct SomeArgument {
    a: TrivialBool,
    b: TrivialEnum<SomeEnum>,
}
```



