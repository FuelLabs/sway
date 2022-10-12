# Comments and Logging

## Comments

Comments in Sway start with two slashes and continue until the end of the line. For comments that extend beyond a single line, you'll need to include `//` on each line.

```sway
// hello world
```

```sway
// let's make a couple of lines
// commented.
```

You can also place comments at the ends of lines containing code.

```sway
fn main() {
    let baz = 8; // Eight is a lucky number
}
```

You can also do block comments

```sway
fn main() {
    /*
    You can write on multiple lines
    like this if you want
    */
    let baz = 8;
}
```

## Logging

The `logging` library provides a generic `log` function that can be imported using `use std::logging::log` and used to log variables of any type. Each call to `log` appends a `receipt` to the list of receipts. There are two types of receipts that a `log` can generate: `Log` and `LogData`.

### `Log` Receipt

The `Log` receipt is generated for _non-reference_ types, namely `bool`, `u8`, `u16`, `u32`, and `u64`. For example, logging an integer variable `x` that holds the value `42` using `log(x)` may generate the following receipt:

```console
"Log": {
  "id": "0000000000000000000000000000000000000000000000000000000000000000",
  "is": 10352,
  "pc": 10404,
  "ra": 42,
  "rb": 1018205,
  "rc": 0,
  "rd": 0
}
```

Note that `ra` will include the value being logged. The additional registers `rc` and `rd` will be zero when using `log` while `rb` may include a non-zero value representing a unique ID for the `log` instance. The unique ID is not meaningful on its own but allows the Rust and the TS SDKs to know the type of the data being logged, by looking up the log ID in the JSON ABI file.

### `LogData` Receipt

`LogData` is generated for _reference_ types which include all types except for the _non_reference_ types mentioned above. For example, logging a `b256` variable `b` that holds the value `0x1111111111111111111111111111111111111111111111111111111111111111` using `log(b)` may generate the following receipt:

```console
"LogData": {
  "data": "1111111111111111111111111111111111111111111111111111111111111111",
  "digest": "02d449a31fbb267c8f352e9968a79e3e5fc95c1bbeaa502fd6454ebde5a4bedc",
  "id": "0000000000000000000000000000000000000000000000000000000000000000",
  "is": 10352,
  "len": 32,
  "pc": 10444,
  "ptr": 10468,
  "ra": 0,
  "rb": 1018194
}
```

Note that `data` in the receipt above will include the value being logged as a hexadecimal. Similarly to the `Log` receipt, additional registers are written: `ra` will always be zero when using `log`, while `rb` will contain a unique ID for the `log` instance.

> **Note**
> The Rust SDK exposes [APIs](https://fuellabs.github.io/fuels-rs/master/calling-contracts/logs.html#logs) that allow you to retreive the logged values and display them nicely based on their types as indicated in the JSON ABI file.
