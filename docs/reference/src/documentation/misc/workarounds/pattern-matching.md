# Pattern Matching

## Nested Match Expressions

In [nested match expressions](../../language/control-flow/match/complex/nested-expression.md) we nest a `match` expression by embedding it inside the `{}` brackets on the right side of the arrow `=>`.

Match expressions cannot be used as a pattern, the left side of the arrow `=>`.

## Constants

When matching on [constants](../../language/control-flow/match/complex/constant.md) we specify that a constant must be used in order to match on a variable. Dynamic values, such as an argument to a function, cannot be matched upon because it will be treated as a [`catch_all`](../../language/control-flow/match/single-line.md) case and thus any subsequent patterns will not be checked.
