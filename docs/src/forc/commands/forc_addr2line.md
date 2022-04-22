# forc-addr2line
Show location and context of an opcode address in its source file


## USAGE:
forc addr2line [OPTIONS] --sourcemap-path <SOURCEMAP_PATH> --opcode-index <OPCODE_INDEX>


## OPTIONS:

`-c`, `--context` <_CONTEXT_>

How many lines of context to show [default: 2]

`-g`, `--sourcemap-path` <_SOURCEMAP_PATH_>

Source file mapping in JSON format

`-h`, `--help` 

Print help information

`-i`, `--opcode-index` <_OPCODE_INDEX_>

Opcode index

`-s`, `--search-dir` <_SEARCH_DIR_>

Where to search for the project root [default: .]