# The Sway Programming Language

Welcome to the Sway programming language book 🌴.

**Q: Hi! What is Sway?**

Sway is a domain-specific programming language for implementing smart contracts on blockchain platforms, most notably for the [Fuel Virtual Machine (Fuel VM)](https://docs.fuel.network/docs/specs/fuel-vm/).

Heavily inspired by [Rust](https://doc.rust-lang.org/book/)'s approach to systems programming, Sway aims to bring modern programming language features and tooling to smart contract development whilst retaining performance, fine grained control and making extensive use of static analysis to prevent common security issues.

**Q: What does "domain-specific" mean?**

Sway is specifically made to be used within a blockchain environment, which behaves very differently than traditional computers.
This domain specific design permits it to make the right decisions about trade-offs at every level of the stack, enabling you to write fast, secure and cost effective smart contracts with features suited to your specific needs.

**Q: Why not use Solidity?**

Solidity is a venerable pioneer but it suffers from being tied to a lot of the historical quirks of the EVM.
It lacks common features programmers have come to expect, has a relatively inexpressive type system, and it lacks a unified tooling ecosystem.

In Sway, we let you design smart contracts with a full modern box of tools.
You get a fully featured language with generics, algebraic types and trait based polymorphism.
You also get an integrated, unified and easy to use toolchain with code completion LSP server, formatter, documentation generation and everything you need to run and deploy your contracts so that nothing comes between you and implementing what you want.

Our expressive type system allows you to catch semantic mistakes, we provide good defaults and we do extensive static analysis checks (such as enforcing the [Checks, Effects, Interactions](./blockchain-development/calling_contracts.md#cei-pattern-violation-static-analysis) pattern) so that you can make sure you write secure and correct code at compile time.

**Q: Why not use Rust?**

Whilst Rust is a great systems programming language (and Sway itself is written in Rust), it isn't suited for smart contract development.

Rust shines because it can use zero-cost abstractions and its sophisticated borrow-checker memory model to achieve impressive runtime performance for complex programs without a garbage collector.

On a blockchain, cost of execution and deployment is the scarce resource.
Memory usage is low and execution time is short.
This makes complex memory management in general much too expensive to be worthwhile and Rust's borrow checker a burden with no upside.

General purpose programming languages in general are ill suited to this environment because their design has to assume execution on a general-purpose computing environment.

Sway attempts to bring all the other advantages of Rust, including its modern type system, approach to safety and good defaults to smart contract developers by providing familiar syntax and features adapted to the specific needs of the blockchain environment.

**Q: I don't know Rust or Solidity. Can I still learn Sway?**

Yes! If you are familiar with the basics of programming, blockchain, and using a terminal you can build with Sway.

**Q: What can I build with Sway?**

You can build smart contracts and their components and libraries for them.
You can learn more about the different program types and how they fit together in the [Program Types](./sway-program-types/index.md) section.

**Q: Do I need to install anything?**

If you want to develop with Sway in your local environment, you need to install [`fuelup`](https://docs.fuel.network/guides/installation/) and your editor of choice that supports LSP, such as [VSCode](https://code.visualstudio.com/).

If you don't want to install anything just yet, you can use the [Sway Playground](https://www.sway-playground.org/) to edit, compile, and deploy Sway code.

**Q: Where can I find example Sway code?**

You can find example applications built with Sway in the [Sway Applications repository](https://github.com/FuelLabs/sway-applications) on GitHub. You can also find projects building on Fuel in the [Fuel ecosystem home](https://app.fuel.network/ecosystem).

**Q: What is the standard library?**

The [standard library](./introduction/standard_library.md), also referred to as `std`, is a library that offers core functions and helpers for developing in Sway. The standard library has its own [reference documentation](https://fuellabs.github.io/sway/master/std/) that has detailed information about each module in `std`.

**Q: What are Sway standards?**

Similar to ERC standards for Ethereum and Solidity, Sway has its own SRC standards that help enable cross compatibility across different smart contracts. For more information on using a Sway Standard, you can check out the [Sway-Standards Repository](https://github.com/FuelLabs/sway-standards).

**Q: How can I make a token?**

Sway has multiple native assets. To mint a new native asset, check out the [native assets](./blockchain-development/native_assets.md) page.

**Q: How can I make an NFT?**

You can find an example of an NFT contract in Sway in the [Sway Applications repo](https://github.com/FuelLabs/sway-applications/tree/master/NFT).

**Q: How can I test Sway code?**

Sway provides [unit testing](./testing/unit-testing.md), so you can test your Sway code with Sway. You can also use the Fuel [Rust SDK](https://docs.fuel.network/docs/fuels-rs/testing/) or [TypeScript SDK](https://docs.fuel.network/docs/fuels-ts/testing/) to test your Sway programs.

**Q: How can I deploy a contract?**

You can use the `forc deploy` command to deploy a contract. For a detailed guide on how to deploy a contract, refer to the [quickstart guide](https://docs.fuel.network/docs/intro/quickstart-contract/).

**Q: Is there a way to convert Solidity code to Sway?**

Yes! You can use the Solidity to Sway transpiler built in to the [Sway Playground](https://www.sway-playground.org/) to convert Solidity code into Sway code. Note that the transpiler is still experimental, and may not work in every case.

**Q: How can I get help with Sway?**

If you run into an issue or have a question, post it on the [Fuel forum](https://forum.fuel.network/) so someone in the Fuel community can help.

**Q: Where should I get started?**

*Ready to build?* You can find step-by-step guides for how to build an application with Sway in the [Fuel Developer Guides](https://docs.fuel.network/guides/).

*Want to read?* Get started by reading the [Introduction](./introduction/index.md) and [Basics](./basics/index.md) sections of this book.
