# Loops

## While

Loops in Sway are currently limited to `while` loops. This is what they look like:

```sway
while counter < 10 {
    counter = counter + 1;
}
```

You need the `while` keyword, some condition (`value < 10` in this case) which will be evaluated each iteration, and a block of code inside the curly braces (`{...}`) to execute each iteration.

### Break & Continue

There are no `break` or `continue` keywords yet, but they're coming.

For now, the way to break out of a `while` loop early is to manually invalidate the condition. In this case, that just means setting `counter` to be >= 10.

Building on the previous example, here's what that might look like:

```sway
let mut break_early = false;
while counter < 10 {
    if break_early == true {
        // here we ensure the condition will evaluate to false, breaking the loop
        counter = 10
    } else {
        // calling some other function to set the bool value
        break_early = get_bool_value();
        counter = counter + 1
    }
}
```

### Nested loops

You can also use nested `while` loops if needed:

```sway
while condition_1 == true {
    // do stuff...
    while condition_2 == true {
        // do more stuff...
    }
}
```
