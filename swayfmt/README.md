# _swayfmt_

A tool for formatting Sway code according to style guidelines.

## Quickstart

- To use `swayfmt`, install the [forc-fmt](../forc-plugins/forc-fmt/) plugin using [fuelup](https://github.com/FuelLabs/fuelup).
- To contribute to `swayfmt`, see [CONTRIBUTING](./CONTRIBUTING.md).

## Configuration with `swayfmt.toml`

Swayfmt _is_ meant to be configurable, however currently it only supports the default configuration options.

> **Note:** Not all `Config` options are guaranteed to work
> and most are not implemented. Configuration options are
> subject to change between versions without notice.
> If you would like to see a feature implemented, or
> implement a feature please refer to the [CONTRIBUTING guide](./CONTRIBUTING.md).

The default `swayfmt.toml`:

```toml
max_width = 100
hard_tabs = false
tab_spaces = 4
newline_style = Auto
indent_style = Block
newline_threshold = 1
group_imports = Preserve
imports_granularity = Preserve
imports_indent = Block
reorder_imports = true
reorder_modules = true
reorder_impl_items = false
item_brace_style = SameLineWhere
blank_lines_upper_bound = 1
blank_lines_lower_bound = 0
empty_item_single_line = true
format_strings = false
hex_literal_case = Preserve
expr_brace_style = AlwaysSameLine
trailing_semicolon = true
space_before_colon = false
space_after_colon = false
type_combinator_layout = Wide
spaces_around_ranges = false
match_block_trailing_comma = false
match_arm_leading_pipe = Never
force_multiline_blocks = false
fn_args_layout = Tall
fn_single_line = false
heuristics_pref = Scaled
use_small_heuristics = true
field_alignment = Off
small_structures_single_line = true
wrap_comments = false
comment_width = 80
normalize_comments = false
```
