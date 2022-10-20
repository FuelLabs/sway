# Address Namespace

Sway utilizies namespaces to distinguish between address types.

Having multiple address types enforces type-safety and expands the range of values that an address can take because the same value can be used across multiple types.

The main types are:

- [`Address`](address.md): Used to identify the UTXO output
- [`ContractId`](contract-id.md): Used to identify a contract

For ease of use there is an [enum](../../language/built-ins/enums.md) wrapper [`Identity`](identity.md) which contains both types.
