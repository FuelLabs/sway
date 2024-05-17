# IR Optimization Passes

Optimization passes are a part of the compiler interface. They can be specified in build configs via their names and their description can be in the compiler output.

To keep the passes consistent in appearance and simple to use from CLI and in build configs, we follow these conventions:
- ideally, if there is an equivalent [LLVM transform pass](https://llvm.org/docs/Passes.html) we take that pass name and short description, but considering the below points.
- use established abbreviations to make pass names short and simple to type. E.g., "fn", "const", "sroa".
- use kebab-case in pass names. E.g., "const-demotion" and not "constdemotion".
- explain passes as nouns in descriptions. E.g., "Function inlining" and not "Inline function".
- the pass description does not end with a dot. E.g., "Constant folding" and not "Constant folding."
- the pass description does not use abbreviations. E.g., "Constant folding" and not "Const folding."

For the sake of our internal consistency, we apply the same conventions to the utility passes like e.g., module printer, and to analysis passes like e.g., escaped symbols.
