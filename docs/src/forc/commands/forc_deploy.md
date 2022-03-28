# Deploy

## SYNOPSIS

`forc deploy` [_options_]

## DESCRIPTION

Deploy contract project. Crafts a contract deployment transaction then sends it to a running node.

Alternatively, you could deploy your contract programmatically using our SDK:

```rust
    // Build the contract
    let salt: [u8; 32] = rng.gen();
    let salt = Salt::from(salt);
    let compiled = Contract::compile_sway_contract("./", salt).unwrap();

    // Launch a local network and deploy the contract
    let client = Provider::launch(Config::local_node()).await.unwrap();
    let contract_id = Contract::deploy(&compiled, &client).await.unwrap();
```

## OPTIONS

`-g`, `--debug-outfile` _debug_outfile_

If set, outputs source file mapping in JSON format

`--minify-json-abi`

By default the JSON for ABIs is formatted for human readability. By using this option JSON output will be "minified", i.e. all on one line without whitespace

`-o` _binary_outfile_

If set, outputs a binary file representing the script bytes

`--offline`

Offline mode, prevents Forc from using the network when managing dependencies. Meaning it will only try to use previously downloaded dependencies

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

`-s`, `--silent`

Silent mode. Don't output any warnings or errors to the command line

`--use-ir`

Whether to compile using the IR pipeline
