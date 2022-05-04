# Escrow

The following code is an example of what an Escrow contract may look like in Sway.

To fetch a template run

```bash
forc init --template escrow <name_of_your_escrow_project>
```

It consists of 4 functions
- constructor
- deposit
- approve
- withdraw

---
## constructor

The `constructor()` is a gatekeeper that is meant to initialize the contract with a
few variables namely
- The two parties involved (called `buyer` & `seller`)
- The asset that is meant to be used for depositing (`asset`)
- The amount of asset required for the deposit (`asset_amount`) A.K.A the "price"

To prevent reinitialization, which may lead to unexpected consequences if subsequent
functions are called, there is a `storage.state` update which unlocks the remaining
functions and blocks the `constructor()` from being called again.

---

## deposit

The `deposit()` function accepts calldata that must match the state stored in storage
(when the contract was initialized via the constructor) and it can only be called after the constructor has been initialized.

This means that
- Only the `buyer` & `seller` can successfully call into the contract
- If they pass in the correct
    - Asset
    - Amount of asset

There is also a catch to prevent the `buyer` or `seller` from depositing more than
once because the `approve()` function transfers back the `storage.price`.


**NOTE**

We cannot stop forceful transfers to any part of the contract meaning
it is possible to get your asset stuck in the contract. This is why we return the
`storage.price` for the specified `asset` in `approve()` as that prevents the contract from possibly being drained of random funds.

---

## approve

The `approve()` function is unlocked after the `constructor()` has been initialized however it will panic if called prior to a _deposit_.

Once the `buyer` and `seller` have deposited approval from both parties will result
in the transfer of `storage.asset` back to them in `storage.asset_amount`.

The final part of the process is to permanently lock the contract into a "completed"
state so that no further logic can be executed.

---

## withdraw

The `withdraw()` function is meant to allow the `buyer` and `seller` to withdraw their deposit at any point prior to the locking of the contract.

---

# Contract Code

```sway
{{#include ../../../examples/escrow/src/main.sw}}
```
