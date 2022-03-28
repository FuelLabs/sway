# Run

## SYNOPSIS

`forc run` [_options_] [_node_url_]

## DESCRIPTION

Run script project. Crafts a script transaction then sends it to a running node.

## OPTIONS

`--contract` _contract_

32-byte contract ID that will be called during the transaction

`-d`, `--data` _data_

Hex string of data to input to script

`--dry-run`

Only craft transaction and print it out

`-g`, `--debug-outfile` _debug_outfile_

If set, outputs source file mapping in JSON format

`-k`, `--kill-node`

Kill Fuel Node Client after running the code. This is only available if the node is started from `forc run`

`--minify-json-abi`

By default the JSON for ABIs is formatted for human readability. By using this option JSON output will be "minified", i.e. all on one line without whitespace

`-o` _binary_outfile_

If set, outputs a binary file representing the script bytes

`--output-directory` _output_directory_

The directory in which the sway compiler output artifacts are placed.

By default, this is `<project-root>/out`.

`-p`, `--path` _path_

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

`--use-ir`

Whether to compile using the IR pipeline
