<!-- markdownlint-disable MD041 -->

## EXAMPLE

Compile the sway files of the current project.

```console
$ forc build
Compiled script "my-fuel-project".
Bytecode size is 28 bytes.
```

The output produced will depend on the project's program type. Building script, predicate and contract projects will produce their bytecode in binary format `<project-name>.bin`. Building contracts and libraries will also produce the public ABI in JSON format `<project-name>-abi.json`.

By default, these artifacts are placed in the `out/` directory.

If a `Forc.lock` file did not yet exist, it will be created in order to pin each of the dependencies listed in `Forc.toml` to a specific commit or version.
