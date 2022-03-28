# Addr2line

## SYNOPSIS

`forc addr2line` [_options_] --sourcemap-path <_sourcemap_path_> --opcode-index <_opcode_index_>

## DESCRIPTION

Shows location and context of an opcode address in its source file.

## OPTIONS

`-c`, `--context` <_context_>

How many lines of context to show [default: 2]

`-g`, `--sourcemap-path` <_sourcemap_path_>

Source file mapping in JSON format

`-i`, `--opcode-index` <_opcode_index_>

Opcode index

`-s`, `--search-dir` <_search_dir_>

Where to search for the project root [default: .]
