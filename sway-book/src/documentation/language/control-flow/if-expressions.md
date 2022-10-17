# if expressions

Sway supports _if_, _else_, and _else if_ expressions which provide control over which instructions should be executed depending on the conditions.

## Conditional Branching

In the following example we have a hardcoded variable `number` set to the value of `5` which is put through some conditional checks.

```sway
{{#include ../../../code/language/control_flow/src/lib.sw:conditional}}
```

The conditional checks are performed in the order that they are defined therefore the first check is to see if the `number` is divisible by `3`. 

If the condition evaluates to the boolean value of `true` then we call `function 1` and we move on to the end where the comment `more code here` is written. We do not evaluate the remaining conditions.

On the other hand if the condition evaluates to `false` then we check the next condition, in this case if the `number` is divisible by `4`. We can have as many `else if` checks as we like as long as they evaluate to a boolean.

At the end there is a special case which is known as a `catch all` case i.e. the `else`. What this means is that we have gone through all of our conditional checks above and none of them have been met. In this scenario we may want to have some special logic to handle a generic case which encompases all the other conditions which we do not care about or can be treated in the same way.

## Using if & let together

In [Conditional Branching](#conditional-branching) we have opted to call some functions depending on which condition is met however that is not the only thing that we can do. Since `if`'s are expressions in Sway we can use them to match on a pattern.


### if let

In the following example we combine `if` and `let` into `if let` followed by some comparison which must evaluate to a boolean.

```sway
{{#include ../../../code/language/control_flow/src/lib.sw:if_let_enum}}
```

#### Example 1

Here we check to see if the hardcoded variable `one` is the same as the first variant of `Foo`.

```sway
{{#include ../../../code/language/control_flow/src/lib.sw:if_let_example1}}
```

#### Example 2

Alternatively, we can take the outcome of the comparison and assign it directly to a variable.

```sway
{{#include ../../../code/language/control_flow/src/lib.sw:if_let_example2}}
```

The syntax above can be altered to include an `else if`.
