# Time Library

The `std::time` library provides utilities for handling time durations and timestamps in Sway smart contracts.

## Duration

Represents a span of time in seconds.

### Creating Durations

```sway
{{#include ../../../../examples/time/src/main.sw:create_durations}}
```

### Converting Durations

Time `std::time` library supports conversion between different time scales such as `seconds`, `minutes`, `hours`, `days`, and `weeks`.

```sway
{{#include ../../../../examples/time/src/main.sw:convert_durations}}
```

### Operations

The `std::time` supports operations on the `Duration` type.

```sway
{{#include ../../../../examples/time/src/main.sw:duration_operations}}
```

## Time

Represents a UNIX timestamp (seconds since Jan 1, 1970).

### Creating Timestamps

There are 3 major ways to create a new timestamp.

```sway
{{#include ../../../../examples/time/src/main.sw:create_timestamps}}
```

### Time Operations

Operations on the `Time` type are supported with conjunction of the `Duration` type.

```sway
{{#include ../../../../examples/time/src/main.sw:time_operations}}
```

### TAI64 Conversion

The Fuel VM internally uses TAI64 time. Conversions between UNIX and TAI64 are maintained with the `Time` type.

```sway
{{#include ../../../../examples/time/src/main.sw:tai64_conversion}}
```

### TAI64 vs UNIX Time

#### Conversion Details

The library uses:

```sway
const TAI_64_CONVERTER: u64 = 10 + (1 << 62);
```

(1 << 62) (0x4000000000000000) marks value as TAI64. 10 accounts for initial TAI-UTC offset in 1970.

Conversion formulas:

`UNIX → TAI64: tai64 = unix + TAI_64_CONVERTER`

`TAI64 → UNIX: unix = tai64 - TAI_64_CONVERTER`

#### Key Differences

| Feature      | TAI64                    | UNIX                      |
|--------------|--------------------------|---------------------------|
| Epoch        | 1970-01-01 00:00:00 TAI  | 1970-01-01 00:00:00 UTC   |
| Leap Seconds | No leap seconds          | Includes leap seconds     |
| Stability    | Continuous time scale    | Discontinuous adjustments |
| Value Range  | (1 << 62) + offset (10s) | Seconds since epoch       |

#### Why TAI64?

* Deterministic execution: No leap second ambiguities
* Monotonic time: Always increases steadily
* Blockchain-friendly: Aligns with Fuel's timestamp mechanism

## Best Practices

1. Use `Duration` for time spans instead of raw seconds
2. Always handle `TimeError` results from `duration_since()` and `elapsed()`
3. Convert to TAI64 when interacting with blockchain primitives
4. Use `Time::block()` for historical time comparisons
5. Prefer duration constants (`SECOND`, `HOUR`, etc.) for readability

## Limitations

1. Durations only support second-level precision
2. Time comparisons are limited to u64 range (584 billion years)
3. No calendar/date functionality (only timestamps)
4. Duration conversions truncate fractional units
