# Converting Types

Below are some common type conversions in Sway:

- [Number Conversions](#number-conversions)
- [String Conversions](#string-conversions)
- [Identity Conversions](#identity-conversions)

## Number Conversions

### Convert to `u256`

```sway
{{#include ../../../../examples/converting_types/src/to_u256.sw:to_u256}}
```

### Convert to `u64`

```sway
{{#include ../../../../examples/converting_types/src/to_u64.sw:to_u64_import}}
```

```sway
{{#include ../../../../examples/converting_types/src/to_u64.sw:to_u64}}
```

### Convert to `u32`

```sway
{{#include ../../../../examples/converting_types/src/to_u32.sw:to_u32_import}}
```

```sway
{{#include ../../../../examples/converting_types/src/to_u32.sw:to_u32}}
```

### Convert to `u16`

```sway
{{#include ../../../../examples/converting_types/src/to_u16.sw:to_u16_import}}
```

```sway
{{#include ../../../../examples/converting_types/src/to_u16.sw:to_u16}}
```

### Convert to `u8`

```sway
{{#include ../../../../examples/converting_types/src/to_u8.sw:to_u8_import}}
```

```sway
{{#include ../../../../examples/converting_types/src/to_u8.sw:to_u8}}
```

### Convert to `Bytes`

```sway
{{#include ../../../../examples/converting_types/src/bytes.sw:to_bytes_import}}
```

```sway
{{#include ../../../../examples/converting_types/src/bytes.sw:to_bytes}}
```

### Convert from `Bytes`

```sway
{{#include ../../../../examples/converting_types/src/bytes.sw:to_bytes_import}}
```

```sway
{{#include ../../../../examples/converting_types/src/bytes.sw:from_bytes}}
```

## String Conversions

### Convert `str` to `str[]`

```sway
{{#include ../../../../examples/converting_types/src/strings.sw:strings_import}}
```

```sway
{{#include ../../../../examples/converting_types/src/strings.sw:str_to_str_array}}
```

### Convert `str[]` to `str`

```sway
{{#include ../../../../examples/converting_types/src/strings.sw:str_array_to_str}}
```

## Identity Conversions

### Convert to `Identity`

```sway
{{#include ../../../../examples/converting_types/src/identity.sw:convert_to_identity}}
```

### Convert `Identity` to `ContractId` or `Address`

```sway
{{#include ../../../../examples/converting_types/src/identity.sw:convert_from_identity}}
```

### Convert `ContractId` or `Address` to `b256`

```sway
{{#include ../../../../examples/converting_types/src/identity.sw:convert_to_b256}}
```

### Convert `b256` to `ContractId` or `Address`

```sway
{{#include ../../../../examples/converting_types/src/identity.sw:convert_b256_to_address_or_contract_id}}
```
