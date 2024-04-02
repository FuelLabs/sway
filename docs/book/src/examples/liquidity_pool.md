# Liquidity Pool Example

All contracts in Fuel can mint and burn their own native asset. Contracts can also receive and transfer any native asset including their own. Internal balances of all native assets pushed through calls or minted by the contract are tracked by the FuelVM and can be queried at any point using the `balance_of` function from the `std` library. Therefore, there is no need for any manual accounting of the contract's balances using persistent storage.

The `std` library provides handy methods for accessing Fuel's native asset operations.

In this example, we show a basic liquidity pool contract minting its own native asset LP asset.

```sway
{{#include ../../../../examples/liquidity_pool/src/main.sw}}
```
