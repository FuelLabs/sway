# General

> TODO: need help filling this in, might remove this page and move content into individual sections

- [Issue: #870](https://github.com/FuelLabs/sway/issues/870)
  - All impl blocks need to be defined before any of the functions they define can be called. This includes sibling functions in the same impl declaration, i.e., functions in an impl can't call each other yet.
- No compiler optimization passes have been implemented yet, therefore bytecode will be more expensive and larger than it would be in production. Note that eventually the optimizer will support zero-cost abstractions, avoiding the need for developers to go down to inline assembly to produce optimal code.
