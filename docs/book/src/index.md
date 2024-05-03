# The Sway Programming Language

Welcome to the Sway programming language book 🌴.

**Q: Hi! What is Sway?**

Sway is a domain-specific language (DSL) for the [Fuel Virtual Machine (Fuel VM)](https://docs.fuel.network/docs/specs/fuel-vm/), a blockchain-optimized VM designed for the Fuel blockchain. Sway is based on [Rust](https://doc.rust-lang.org/book/), and includes syntax to leverage a blockchain VM without needlessly verbose boilerplate.

**Q: What does "domain-specific" mean?**

A domain-specific language is a language that is made and used in a specific environment for a specific purpose, in this case execution in the Fuel VM. This differs from a general-purpose language that can be used for several environments and purposes, like Rust.

**Q: Why doesn't Fuel use Solidity?**

Sway aims to take the best of Solidity, and leave the worst. From Solidity, we took the notion of smart contract programming as a paradigm. This led to storage blocks, contract ABIs as entry points, and more.

While Solidity has been used to make a lot of amazing contracts, as a developer it can also feel like a minefield for vulnerabilities. Instead of having to be a 10x developer to feel confident deploying a contract, Sway takes a lot of the the stress out of developing smart contracts.

Sway provides multiple layers of safety. For one, we provide canonical tooling and "one right way" to do things. This results in less ambiguity and more correct and helpful tools. These tools ensure the programmer has nothing between them and the algorithm they are trying to implement.

In addition, Sway has implemented static analysis checks like a [Checks, Effects, Interactions](./blockchain-development/calling_contracts.md#cei-pattern-violation-static-analysis) checker, state and storage purity checking, immutable-by-default semantics, and other static compile-time audits to promote safety.

**Q: Why doesn't Fuel use Rust?**

Sway aims to take the best of Rust, and leave out everything that doesn't make sense in blockchain.

From Rust, we took the prioritizations of performance, control, and safety. In Rust, this means the borrow checker, safe parallelism (send and sync), annotated unsafety, etc., mainly to save the programmer from referencing freed memory, shared mutable state, and undesirable memory management.

This is great for a general-purpose language model. Sway, however, is not general purpose. Sway targets a blockchain VM environment, where execution is not indefinite, and memory allocation and management are less of a concern. Instead, we need to optimize for gas costs and contract-level safety.

We applied the philosophy of performance, control, and safety and interpreted it in this new context. This is where Sway gets compile time checks of state mutability, namespacing of state variables, static analysis passes, and gas optimization.

**Q: I don't know Rust or Solidity. Can I still learn Sway?**

Yes! If you are familiar with the basics of programming, blockchain, and using a terminal you can build with Sway.

**Q: What can I build with Sway?**

There are four types of programs you can build with Sway: contracts, scripts, predicates, and libraries. You can learn more about each of these in the [Program Types](./sway-program-types/index.md) section.

**Q: Do I need to install anything?**

If you want to develop with Sway in your local environment, you need to install [`fuelup`](https://docs.fuel.network/guides/installation/) and have an IDE such as [VSCode](https://code.visualstudio.com/).

If you don't want to install anything just yet, you can use the [Sway Playground](https://www.sway-playground.org/) to edit, compile, and deploy Sway code.

**Q: Where can I find example Sway code?**

You can find example applications built with Sway in the [Sway Applications repository](https://github.com/FuelLabs/sway-applications) on GitHub. You can also find projects building on Fuel in the [Fuel ecosystem home](https://app.fuel.network/ecosystem).

**Q: What is the standard library?**

The [standard library](./introduction/standard_library.md), also referred to as the `std-lib`, is a library that offers core functions and helpers for developing in Sway. The standard library has it's own [reference documentation](https://fuellabs.github.io/sway/master/std/) that has detailed information about each module in the `std-lib`.

**Q: What are Sway standards?**

Similar to ERC standards for Ethereum and Solidity, Sway has it's own SRC standards that help enable cross compatibility across different smart contracts. For more information on using a Sway Standard, you can check out the [Sway-Standards Repository](https://github.com/FuelLabs/sway-standards).

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
