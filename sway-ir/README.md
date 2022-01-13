# Sway Intermediate Representation

This crate is a work-in-progress library for providing an [SSA](https://en.wikipedia.org/wiki/Static_single_assignment_form) style IR for the [Sway](https://github.com/FuelLabs/sway) middle end.

It is modelled after [LLVM](https://llvm.org/docs/LangRef.html) to a degree, and is designed to simplify the optimization phase of the compiler pipeline.

It is currently lacking several features and documentation, not to mention optimization passes, but is already capable of being targeted by Sway for codegen and passes the test suite.
