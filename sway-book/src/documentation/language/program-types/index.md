# Sway Program Types

A Sway program is a file ending with the extension `.sw`, e.g. `main.sw`, and the first line of the file is a decleration of the _type_ of program that it is.

A Sway program can be one of four types: 

- [contract](contract.md)
  - Primarily used for protocols or systems that operate within a fixed set of rules e.g. staking contracts, decentralized exchanges
- [library](library.md)
  - Reusable code for handling common operations
- [script](script.md)
  - Used for complex, multi-step, on-chain interactions that won't persist e.g. using a decentralized exchange to create a leveraged position (borrow, swap, re-collateralize, borrow)
- [predicate](predicate.md)
  - A set of preconditions to the construction of transaction, the result of which must be a boolean value of `true` in order for the transaction to be considered valid

## Sway Project Types

A project type in Sway refers to which program type is in the main file of the project. 

This means that there are four types of projects:

- contracts
- libraries
- scripts
- predicates

All four projects can contain multiple library files in the `src` directory.

There is a caveat when it comes to _contracts_, _scripts_ and _predicates_ and it's as follows:

- A project can at most contain *any one* of a contract, script or predicate.

This means that a project cannot contain more than one contract, one script, one predicate and it cannot mix them together.

## Entry Points

An entry point is the starting point of execution for a program.

Since a library is not directly deployable to the blockchain it does not have an entry point and instead its code is exported for use within other programs.

Unlike the library the contract, script and predicate all have an entry point. The contract exposes an `Application Binary Interface (ABI)` while the script and predicate expose a `main()` function for entry.
