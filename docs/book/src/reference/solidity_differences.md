# Differences From Solidity

This page outlines some of the critical differences between Sway and Solidity, and between the FuelVM and the EVM.

## Underlying Virtual Machine

The underlying virtual machine targeted by Sway is the FuelVM, specified [here](https://github.com/FuelLabs/fuel-specs). Solidity targets the Ethereum Virtual Machine (EVM), specified [here](https://ethereum.github.io/yellowpaper/paper.pdf).

## Word Size

Words in the FuelVM are 64 bits (8 bytes), rather than the 256 bits (32 bytes) of the EVM. Therefore, all primitive integers smaller and including `u64` are stored in registers; `u256`, being bigger than the registers, and hashes (the `b256` type) are not stored in registers but rather in memory. They are therefore pointers to a 32-byte memory region containing their data.

## Unsigned Integers Only

Only unsigned integers are provided as primitives: `u8`, `u16`, `u32`, `u64`, and `u256`. Signed integer arithmetic is not available in the FuelVM. Signed integers and signed integer arithmetic can be implemented in high-level libraries if needed.

## Global Revert

Panics in the FuelVM (called "reverts" in Solidity and the EVM) are global, i.e. they cannot be caught. A panic will completely and unconditionally revert the stateful effects of a transaction, minus gas used.

## Default Safe Math

<!-- This section should explain safe math in Fuel vs EVM -->
<!-- safe_math:example:start -->
Math in the FuelVM is by default safe (i.e. any overflow or exception is a panic). Safety checks are performed natively in the VM implementation, rather than at the bytecode level like [Solidity's default safe math](https://docs.soliditylang.org/en/latest/080-breaking-changes.html#silent-changes-of-the-semantics).
<!-- safe_math:example:end -->

## No* Code Size Limit

There is no practical code size limit to Sway contracts. The physical limit is governed by the [`VM_MAX_RAM` VM parameter](https://fuellabs.github.io/fuel-specs/master/vm#parameters), which at the time of writing is 64 MiB.

## Account Types

Account types in the FuelVM have type-safe wrappers around primitive `b256` hashes to clearly distinguish their respective types. The wrapper `Address` mirrors that of an EOA (Externally Owned Account) and has the ability to hold UTXOs in the context of the EVM. The other wrapper, `ContractId`, reflects that of a deployed contract in the EVM but cannot hold UTXOs.
