# Methods and Associated Functions

<!-- This section should explain methods & associated functions in Sway -->
<!-- methods_af:example:start -->
Methods are similar to functions in that we declare them with the `fn` keyword and they have parameters and return a value. However, unlike functions, _Methods_ are defined within the context of a struct (or enum), and either refers to that type or mutates it. The first parameter of a method is always `self`, which represents the instance of the struct the method is being called on.

_Associated functions_ are very similar to _methods_, in that they are also defined in the context of a struct or enum, but they do not actually use any of the data in the struct and as a result do not take _self_ as a parameter. Associated functions could be standalone functions, but they are included in a specific type for organizational or semantic reasons.

To declare methods and associated functions for a struct or enum, use an _impl block_. Here, `impl` stands for implementation.
<!-- methods_af:example:end -->

```sway
{{#include ../../../../examples/methods_and_associated_functions/src/main.sw}}
```

<!-- This section should explain how to call a method -->
<!-- call_method:example:start -->
To call a method, simply use dot syntax: `foo.iz_baz_true()`.
<!-- call_method:example:end -->

<!-- This section should explain how methods + assoc. fns can accept `ref mut` params -->
<!-- ref_mut:example:start -->
Similarly to [free functions](functions.md), methods and associated functions may accept `ref mut` parameters.
<!-- ref_mut:example:end -->

For example:

```sway
{{#include ../../../../examples/ref_mut_params/src/main.sw:move_right}}
```

and when called:

```sway
{{#include ../../../../examples/ref_mut_params/src/main.sw:call_move_right}}
```
