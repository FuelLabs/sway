# String Type

In Sway, static-length strings are a primitive type. This means that when you declare a string, its size is a part of its type. This is necessary for the compiler to know how much memory to give for the storage of that data. The size of the string is denoted with square brackets. Let's take a look:

```sway
let my_string: str[4] = "fuel";
```

Because the string literal `"fuel"` is four letters, the type is `str[4]`, denoting a static length of 4 characters. Strings default to UTF-8 in Sway.
