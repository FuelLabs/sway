# Contract Ownership

<!-- TODO: ignore, planning on working on this soon, haven't made any adjustments from the current book -->

<!-- Many contracts require some form of ownership for access control. To accomplish this, it is recommended that a storage variable of type `Option<Identity>` is used to keep track of the owner. This allows setting and revoking ownership using the variants `Some(..)` and `None` respectively. This is better, safer, and more readable than using the `Identity` type directly where revoking ownership has to be done using some magic value such as `std::constants::ZERO_B256` or otherwise.

The following is an example of how to properly set ownership of a contract:

```sway
{{#include ../../../examples/ownership/src/main.sw:set_owner_example}}
```

The following is an example of how to properly revoke ownership of a contract:

```sway
{{#include ../../../examples/ownership/src/main.sw:revoke_owner_example}}
``` -->
