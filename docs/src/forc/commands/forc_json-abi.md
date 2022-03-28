# JSON-ABI

## SYNOPSIS

`forc json-abi` [_options_]

## DESCRIPTION

Output the JSON associated with the ABI.

## OPTIONS

`--minify`

By default the JSON for ABIs is formatted for human readability. By using this option JSON output will be "minified", i.e. all on one line without whitespace

`-o` _json_outfile_

If set, outputs a json file representing the output json abi

`--offline`

Offline mode, prevents Forc from using the network when managing dependencies. Meaning it will only try to use previously downloaded dependencies

`-p`, `--path` _path_

Path to the project, if not specified, current working directory will be used

`-s`, `--silent`

Silent mode. Don't output any warnings or errors to the command line
