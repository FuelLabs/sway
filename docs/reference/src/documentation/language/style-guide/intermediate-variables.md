# Intermediate Variables

An intermediate variable, or a temporary variable, is a variable that is typically used once. In most cases we avoid creating intermediate variables; however, there are cases where they may enrich the code.

## Contextual Assignment

It may be beneficial to use an intermediate variable to provide context to the reader about the value.

```sway
{{#include ../../../code/language/style-guide/intermediate_variables/src/lib.sw:contextual_assignment}}
```

## Shortened Name

In the cases of multiple levels of indentation or overly verbose names it may be beneficial to create an intermediate variable with a shorter name. 

```sway
{{#include ../../../code/language/style-guide/intermediate_variables/src/lib.sw:shortened_name}}
```
