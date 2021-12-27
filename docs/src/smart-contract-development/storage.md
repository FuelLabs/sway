# Storage

When developing a [smart contract](TODO: link to our smart contract description), you will typically need some sort of persistant storage. In this case, persistant storage, often just called _storage_ in this context, is a place where you can store values that are persisted inside the contract itself. This is in contrast to a regular value in _memory_, which disappears after the contract exits.

Put in conventional programming terms, contract storage is like saving data to a hard drive. That data is saved even after the program which saved it exits. That data is persistent. Using memory is like declaring a variable in a program: it exists for the duration of the program and is non-persistent.

Some basic use cases of storage include declaring an owner address for a contract and saving balances in a wallet.

## Syntax

### Declaration

The syntax of declaring storage space in Sway looks like this:

```sway
storage {
    owner: b256,
}
```

It is very similar to a struct declaration, except with storage, you also have the option to specify an initial value:

```sway
storage {
    owner: 0xeeb578f9e1ebfb5b78f8ff74352370c120bc8cacead1f5e4f9c74aafe0ca6bfd,
}
```

This value is passed as a part of the transaction, which initializes storage upon contract deployment.

### Access

