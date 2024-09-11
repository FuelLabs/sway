# Features

## Code Actions

_Source:_ [code_actions](https://github.com/FuelLabs/sway/tree/master/sway-lsp/src/capabilities/code_actions)

Quickly generate boilerplate code and code comments for functions, structs, and ABIs.

## Completion

_Source:_ [completion.rs](https://github.com/FuelLabs/sway/blob/master/sway-lsp/src/capabilities/completion.rs)

Suggests code to follow partially written statements for functions and variables.

## Go to Definition

Jumps to the definition of a symbol from its usage.

## Find All References

Locates all occurrences of a symbol throughout the project.

## Hover

_Source:_ [hover](https://github.com/FuelLabs/sway/tree/master/sway-lsp/src/capabilities/hover)

Provides documentation, compiler diagnostics, and reference links when hovering over functions and variables.

## Inlay Hints

_Source:_ [inlay_hints.rs](https://github.com/FuelLabs/sway/blob/master/sway-lsp/src/capabilities/inlay_hints.rs)

Displays the implied type of a variable next to the variable name. Configurable in Settings.

## Rename

_Source:_ [rename.rs](https://github.com/FuelLabs/sway/blob/master/sway-lsp/src/capabilities/rename.rs)

Renames a symbol everywhere in the workspace.

## Diagnostics

_Source:_ [diagnostic.rs](https://github.com/FuelLabs/sway/blob/master/sway-lsp/src/capabilities/diagnostic.rs)

Displays compiler warnings and errors inline.

## Syntax Highlighting

_Source:_ [highlight.rs](https://github.com/FuelLabs/sway/blob/master/sway-lsp/src/capabilities/highlight.rs)

Highlights code based on type and context.

## Run

_Source:_ [runnable.rs](https://github.com/FuelLabs/sway/blob/master/sway-lsp/src/capabilities/runnable.rs)

Shows a button above a runnable function or test.
