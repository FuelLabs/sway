# match

[If expressions](if-expressions.md) can be used to check a large number conditions however there is an alternative syntax which allows us to perform advanced pattern matching.

A `match` expression matches on a variable and checks each case, also known as an `arm`, to see which branch of logic should be performed. 

The cases are checked sequentially in the order they are declared, i.e. from top to bottom, and the last arm must ensure that all cases in the pattern are covered otherwise the compiler will not know how to handle an unspecified pattern and it will error.


