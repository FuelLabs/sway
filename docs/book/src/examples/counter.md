# Counter

The following is a simple example of a contract which implements a counter. Both the `initialize_counter()` and `increment_counter()` ABI methods return the currently set value.

```bash
forc template --template-name counter my_counter_project
```

```sway
{{#include ../../../../examples/counter/src/main.sw}}
```

### Building and deployment

The following commands can be used to build and deploy the contract. For a detailed tutorial, refer to [Building and Deploying](https://docs.fuel.network/guides/contract-quickstart/#building-the-contract).

```bash
# Builds the contract
forc build

# Deploys the contract
forc deploy --testnet
```

It will return you the contract ID.