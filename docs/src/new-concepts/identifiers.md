# Identifiers

Addresses in Sway are similar to Ethereum addresses. The 2 differences are that Sway addresses are 32 bytes long (instead of 20) and are computed with Sha256(PublicKey) (instead of using Keccak256).

Contracts, on the other hand, have a `contract_id` rather than an address.
A contract's id is also 32 bytes long, and is calculated with:
`sha256(0x4655454C ++ tx.data.salt ++ root(tx.data.witnesses[bytecodeWitnessIndex].data))`