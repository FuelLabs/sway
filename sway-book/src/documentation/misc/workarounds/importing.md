# Importing

In [external libraries](../../language/program-types/libraries/external.md) we have looked at how a library can be imported into a project so that code can be reused.

When it comes to importing only external libraries can be imported; any other type of program will result in an error.

This means that the following projects cannot be imported:

- [contracts](../../language/program-types/contract.md)
- [internal libraries](../../language/program-types/libraries/internal.md)
- [scripts](../../language/program-types/script.md)
- [predicates](../../language/program-types/predicate.md)

Contracts can be imported however the workaround is to move the `ABI` into an [external library](../../language/program-types/libraries/external.md) and import that library.

> TODO: alter to show new way of importing contracts?
