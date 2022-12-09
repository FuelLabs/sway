# Comments

There are two kinds of comments in Sway.

- [Regular Comments](#regular-comments) are used for conveying information to the reader of the source code
- [Documentation Comments](#documentation-comments) are used for documenting functionality for external use

## Regular Comments

Regular comments are broken down into two forms of syntax:

- `// comment`
- `/* comment */`

The first form starts after the two forward slashes and continues to the end of the line.

Comments can be placed on multiple lines by starting each line with `//` and they can be placed at the end of some code.

```sway
{{#include ../../../code/language/comments/src/lib.sw:comment}}
```

Similarly, the second form continues to the end of the line and it can also be placed at the end of some code.

```sway
{{#include ../../../code/language/comments/src/lib.sw:block}}
```

## Documentation Comments

Documentation comments start with three forward slashes `///` and are placed on top of functions or above fields e.g. in a [struct](../built-ins/structs.md).

Documentation comments are typically used by tools for automatic documentation generation.

```sway
{{#include ../../../code/language/comments/src/lib.sw:documentation}}
```
