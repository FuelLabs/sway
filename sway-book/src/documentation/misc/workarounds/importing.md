# Importing

In [external libraries](../../language/program-types/libraries/external.md) we have looked at how a library can be imported into a project so that code can be reused.

When it comes to importing only external libraries can be imported through the `Forc.toml` file; any other type of program will result in an error.

This means that the following projects cannot be imported:

- [contracts](../../language/program-types/contract.md)
- [internal libraries](../../language/program-types/libraries/internal.md)
- [scripts](../../language/program-types/script.md)
- [predicates](../../language/program-types/predicate.md)

While contracts cannot be imported, a workaround is to move the contract's `abi` declaration into an [external library](../../language/program-types/libraries/external.md) and import that library anywhere the ABI is needed.

> TODO: move the next comment into a page where it makes sense to keep it

Furthermore, using contract dependencies it is possible to import the contract ID automatically as a public constant.
