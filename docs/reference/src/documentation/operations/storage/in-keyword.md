
# Storage `in` keyword

The `in` keyword can be used within a storage variable to override the position where the value is stored.

## Example

The `in` keyword and position value are used as follows:

```sway
{{#include ../../../../code/operations/storage/storage_in_keyword/src/main.sw:in_keyword}}
```

The code above will force the storage of the variable into the position `HASH_KEY` which is set to `0x7616e5793ef977b22465f0c843bcad56155c4369245f347bcc8a61edb08b7645` instead of the position that would be calculated from the variable name `sha256("storage.current_owners")` which would be `0x84f905e3f560d70fbfab9ffcd92198998ce6f936e3d45f8fcb16b00f6a6a8d7e`
