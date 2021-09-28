# Strategy and Roadmap

## Strategy

### Compiler Architecture 
The Sway compiler employs a typical compiler architecture. The following procedures are performed in the listed order:
1. Parsing
1. Lexing
1. Type checking and inference
1. Control flow analysis and dead code analysis
1. IR generation
1. Optimization passes
1. Bytecode Generation

Each step is modular and independent of subsequent and preceding steps, for the sake of future development and alternate back or front ends.

### Development Strategy
The compiler is developed both as a standalone executable and as a library which is consumed by other tooling for Sway, such as `forc` (including `forc fmt`, `forc doc`, etc.), the language server, and more. The synchronization of the teams working on these tools is key to an integrated, wholistic development experience.

## Roadmap

### Done
1. Smart contracts, scripts, predicates
1. Rust-like compiler with verbose and descriptive errors and warnings
1. Forc package manager and orchestrator (*f*uel-*o*rchestrator)
1. Forc code formatter (`forc fmt`)
1. Compiler test suite
1. Fuel-VM Assembly Expressions
1. Control flow analysis with GraphViz output
1. Visibility into both IR and finalized bytecode
1. Language server and VSCode Plugin
1. Contract calls (`CALL` opcode)
1. Contract ABIs and ABI types


### To be Included in MVP
1. Rust-like Hindley-Milner-based type inference engine
1. Generic types and trait-based inheritence
1. Contract storage access in the standard library
1. Source Maps
1. Auto-generated documentation webpages (`forc doc` -- modeled on `cargo doc` from Rust)


### Post-MVP
1. Safety checks (re-entrancy, safe data types)
1. Macro system
1. Godbolt visualizer
1. Gas profiler
1. More optimization passes
