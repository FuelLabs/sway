# Counter

The following is a simple example of a contract which implements a counter. Both the `initialize_counter()` and `increment_counter()` ABI methods return the currently set value.

```bash
forc template --template-name counter my_counter_project
```

```sway
{{#include ../../../../examples/counter/src/main.sw}}
```

### Building and deployment

the following commands will quickly help you in build and deploy the above contract - for detailed tutorial for this refer to [Building and Deploying](https://docs.fuel.network/guides/contract-quickstart/#building-the-contract). Keep in mind that for deployment you need to have wallet set up already.

```bash
# Builds the contract
forc build

# Deploys the contract
forc deploy --testnet
```

It will return you the contract ID.