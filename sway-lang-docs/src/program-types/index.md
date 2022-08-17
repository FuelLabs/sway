# Sway Program Types

A Sway program is a file ending with the extension `.sw`, e.g. `main.sw`, and the first line of the file is a decleration of the _type_ of program that it is.

A Sway program can be one of four types: 

- contract
- predicate
- script
- library 

The first three are all deployable to the blockchain while a _library_ is a project designed for code reuse and is never directly deployed to the chain.

## Sway Project Types

Since there are four types of programs there can be four types of projects.

A project can be a library which is meant to be imported into other projects. Alternatively, it can be any one of a contract, script or predicate. 

For example, a project can be:

- contract + library
- script + library
- predicate + library
- library

> **NOTE** The libraries are optional for the first 3 bullet points

That is to say that a project cannot consist of multiple contracts / scripts / predicates or any mixture of those.

> **TODO** Exception being multiple contracts in 1 file?

## Entry Points

An entry point is the starting point of execution for a program.

Since a library is not directly deployable to the blockchain it does not have an entry point and instead its code is exported for use within other programs.

Unlike the library the contract, script and predicate all have an entry point. The contract exposes an `Application Binary Interface (ABI)` while the script and predicate expose a `main()` function for entry.

## Use Cases

> **TODO** Use cases do not fit on this page imo, remove?

Contracts are used primarily for protocols or systems that operate within a fixed set of rules. A good example would be a staking contract or a decentralized exchange.

Scripts are used for complex on-chain interactions that won't persist. An example of this may be using a DEX and Lender to create a leveraged position (borrow, swap, re-collateralize, borrow) which is a complex transaction that would usually take multiple steps.

Libraries are for code that is reusable and useful for handling common situations. A good example of this would be a library to handle fixed-point math or big number math.
