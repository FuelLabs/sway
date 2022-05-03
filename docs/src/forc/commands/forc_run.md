# forc-run
Run script project. Crafts a script transaction then sends it to a running node


## USAGE:
forc run [OPTIONS] [NODE_URL]


## ARGS:

<_NODE_URL_>
URL of the Fuel Client Node

[env: FUEL_NODE_URL=]
[default: http://127.0.0.1:4000]


## OPTIONS:

`--byte-price` <_BYTE_PRICE_>


Set the transaction byte price. Defaults to 0


`--contract` <_CONTRACT_>


32-byte contract ID that will be called during the transaction


`-d`, `--data` <_DATA_>


Hex string of data to input to script


`--dry-run` 


Only craft transaction and print it out


`-g`, `--debug-outfile` <_DEBUG_OUTFILE_>


If set, outputs source file mapping in JSON format


`--gas-limit` <_GAS_LIMIT_>


Set the transaction gas limit. Defaults to the maximum gas limit


`--gas-price` <_GAS_PRICE_>


Set the transaction gas price. Defaults to 0


`-h`, `--help` 


Print help information


`-k`, `--kill-node` 


Kill Fuel Node Client after running the code. This is only available if the node is
started from `forc run`


`--minify-json-abi` 


By default the JSON for ABIs is formatted for human readability. By using this option
JSON output will be "minified", i.e. all on one line without whitespace


`-o` <_BINARY_OUTFILE_>


If set, outputs a binary file representing the script bytes


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


`-r`, `--pretty-print` 


Pretty-print the outputs from the node


`-s`, `--silent` 


Silent mode. Don't output any warnings or errors to the command line


`--use-orig-asm` 


Whether to compile using the original (pre- IR) pipeline


`--use-orig-parser` 


Whether to compile using the original (pest based) parser