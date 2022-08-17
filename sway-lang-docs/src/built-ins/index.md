# Built-in Types

Every value in Sway is of a certain type. Although deep down, all values are just ones and zeroes in the underlying virtual machine, Sway needs to know what those ones and zeroes actually mean. This is accomplished with _types_.

Sway is a statically typed language. At compile time, the types of every value must be known. This does not mean you need to specify every single type: usually, the type can be reasonably inferred by the compiler.
