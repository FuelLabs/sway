# Converting Types

Below are some common type conversions in Sway:

- [Identity Conversions](#identity-conversions)
- [String Conversions](#string-conversions)
- [Number Conversions](#number-conversions)
- [Byte Array Conversions](#byte-array-conversions)

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

## Byte Array Conversions

### Convert to a Byte Array

```sway
{{#include ../../../../examples/converting_types/src/byte_arrays.sw:to_byte_array_import}}
```

```sway
{{#include ../../../../examples/converting_types/src/byte_arrays.sw:to_byte_array}}
```

### Convert from a Byte Array

```sway
{{#include ../../../../examples/converting_types/src/byte_arrays.sw:to_byte_array_import}}
```

```sway
{{#include ../../../../examples/converting_types/src/byte_arrays.sw:from_byte_array}}
```
