# forc-build
Compile the current or target project.

The output produced will depend on the project's program type. Building script, predicate and
contract projects will produce their bytecode in binary format `<project-name>.bin`. Building
contracts and libraries will also produce the public ABI in JSON format `<project-name>-abi.json`.


## USAGE:
forc build [OPTIONS]


## OPTIONS:

`-g`, `--debug-outfile` <_DEBUG_OUTFILE_>


If set, outputs source file mapping in JSON format


`-h`, `--help` 


Print help information


`--minify-json-abi` 


By default the JSON for ABIs is formatted for human readability. By using this option
JSON output will be "minified", i.e. all on one line without whitespace


`-o` <_BINARY_OUTFILE_>


If set, outputs a binary file representing the script bytes


`--offline` 


Offline mode, prevents Forc from using the network when managing dependencies. Meaning
it will only try to use previously downloaded dependencies


`--output-directory` <_OUTPUT_DIRECTORY_>


The directory in which the sway compiler output artifacts are placed.

By default, this is `<project-root>/out`.


`-p`, `--path` <_PATH_>


Path to the project, if not specified, current working directory will be used


`--print-finalized-asm` 


Whether to compile to bytecode (false) or to print out the generated ASM (true)


`--print-intermediate-asm` 


Whether to compile to bytecode (false) or to print out the generated ASM (true)


`--print-ir` 


Whether to compile to bytecode (false) or to print out the generated IR (true)


`-s`, `--silent` 


Silent mode. Don't output any warnings or errors to the command line


`--use-orig-asm` 


Whether to compile using the original (pre- IR) pipeline


`--use-orig-parser` 


Whether to compile using the original (pest based) parser

## EXAMPLES:

Compile the sway files of the current project.

```console
$ forc build
Compiled script "my-fuel-project".
Bytecode size is 28 bytes.
```

The output produced will depend on the project's program type. Building script, predicate and contract projects will produce their bytecode in binary format `<project-name>.bin`. Building contracts and libraries will also produce the public ABI in JSON format `<project-name>-abi.json`.

By default, these artifacts are placed in the `out/` directory.

If a `Forc.lock` file did not yet exist, it will be created in order to pin each of the dependencies listed in `Forc.toml` to a specific commit or version.