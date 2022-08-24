# Functions, methods, and associated functions

Functions, and by extension methods and associated functions, are a way to group functionality together in a way that allows for code reuse without having to re-write the code in each place that it is used.

The distinction between a function, method and associated function is as follows:

- A [function](function.md) is a group of code that is independent of any object
- A [method](method.md) is a function that is associated with an object and it uses `self` as the first parameter
- An [associated function](associated-function.md) is a method but without the `self` parameter

> The distinction in the terminology is not that important and most people may refer to all three as "functions" since they all group code together and are defined in the same way.

## Function Decleration

Here is a template that applies to the aforementioned functions.

```sway
<optional pub keyword> <fn keyword> <function name>(<parameter name>: <parameter type>, ...) -> <return type> {
    // function code
}
```

Check out the subsequent pages for concrete examples!
