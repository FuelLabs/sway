# Sway Program Types

A Sway program is a file ending with the extension `.sw`, e.g. `main.sw`, and the first line of the file is a declaration of the _type_ of program.

A Sway program can be one of four types:

- [contract](contract.md)
  - Primarily used for protocols or systems that operate within a fixed set of rules e.g. staking contracts, decentralized exchanges, etc.
- [library](libraries/index.md)
  - Reusable code for handling common operations
- [script](script.md)
  - Used for complex, multi-step, on-chain interactions that won't persist, such as using a decentralized exchange to create a leveraged position (borrow, swap, re-collateralize)
- [predicate](predicate.md)
  - A set of preconditions to the construction of a transaction, the result of which must be a Boolean value of `true` in order for the transaction to be considered valid

## Sway Project Types

A project type in Sway refers to which program type is in the main file of the project.

This means that there are four types of projects:

- contracts
- libraries
- scripts
- predicates

All four projects can contain multiple library files in the `src` directory.

There is a caveat when it comes to _contracts_, _scripts_ and _predicates_ and it's as follows:

- A project can at most contain _any one_ of a contract, script or predicate.

This means that a project cannot contain more than one contract, one script, one predicate and it cannot mix them together.

## Entry Points

An entry point is the starting point of execution for a program.

Since a library is not directly deployable to the blockchain it does not have an entry point and instead its code is exported for use within other programs.

Unlike libraries; contracts, scripts and predicates all have an entry point. Contracts expose an `Application Binary Interface (ABI)` while scripts and predicates expose a `main()` function for entry.
