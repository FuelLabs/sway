# Asset Operations

A common application of a smart contract is the creation of an asset / token i.e. a cryptocurrency.

Managing a cryptocurrency is typically done via the following models:

- Account based e.g. Ethereum
- Unspent Transaction Output (UTXO) e.g. Bitcoin

Sway operates on the UTXO model therefore assets can be transferred out of the contract that created them. What this means is that keeping track of assets that have been transferred out of the contract may be more difficult because the information is not centralized in one place.

With that regard in mind, the account based approach can be partially replicated while utilizing certain asset operations that are build into the FuelVM.

The following sections will take a look at how an asset can be:

- [`Minted`](mint.md) (created)
- [`Burned`](burn.md) (destroyed)
- [`Transferred`](transfer/index.md) (sent)

While also taking a look at:

- [`The contract balance`](balance.md)
