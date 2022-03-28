# Sway Program Types

A Sway program itself has a type: it is either a _contract_, a _predicate_, a _script_, or a _library_. The first three of these things are all deployable to the blockchain. A _library_ is simply a project designed for code reuse and is never directly deployed to the chain.

Every Sway file _must_ begin with a declaration of what type of program it is. A project can have many libraries within it, but only one contract, script, or predicate. Scripts and predicates require `main` functions to serve as entry points, while contracts instead publish an ABI. This chapter will go into detail about all of these various types of programs and what purposes they serve.

Contracts are used primarily for protocols or systems that operate within a fixed set of rules. A good example would be a staking contract or a decentralized exchange.

Scripts are used for complex on-chain interactions that won't persist. An example of this may be using a DEX and Lender to create a leveraged position (borrow, swap, re-collateralize, borrow) which is a complex transaction that would usually take multiple steps.

Libraries are for code that is reusable and useful for handling common situations. A good example of this would be a library to handle fixed-point math or big number math.

- [Contracts](./smart_contracts.md)
- [Libraries](./libraries.md)
- [Scripts](./scripts.md)
- [Predicates](./predicates.md)
