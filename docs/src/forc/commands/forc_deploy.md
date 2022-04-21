# forc-deploy
Deploy contract project. Crafts a contract deployment transaction then sends it to a running node


## USAGE:
forc deploy [OPTIONS]


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


Whether to compile to bytecode (false) or to print out the IR (true)


`-s`, `--silent` 


Silent mode. Don't output any warnings or errors to the command line


`--use-orig-asm` 


Whether to compile using the original (pre- IR) pipeline


`--use-orig-parser` 


Whether to compile using the original (pest based) parser

## EXAMPLES:

You can use `forc deploy`, which triggers a contract deployment transaction and sends it to a running node.

Alternatively, you can deploy your Sway contract programmatically using [fuels-rs](https://github.com/FuelLabs/fuels-rs), our Rust SDK.

You can find an example within our [fuels-rs book](https://fuellabs.github.io/fuels-rs/latest/getting-started/basics.html#deploying-a-sway-contract).