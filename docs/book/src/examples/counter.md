# Counter

The following is a simple example of a contract which implements a counter. Both the `initialize_counter()` and `increment_counter()` ABI methods return the currently set value.

```bash
forc template --template-name counter my_counter_project
```

```sway
{{#include ../../../../examples/counter/src/main.sw}}
```

## Build and deploy

The following commands can be used to build and deploy the contract. For a detailed tutorial, refer to [Building and Deploying](https://docs.fuel.network/guides/contract-quickstart/#building-the-contract).

```bash
# Build the contract
forc build

# Deploy the contract
forc deploy --testnet
```
