# Comments

Comments in Sway start with two slashes and continue until the end of the line. For comments that extend beyond a single line, you'll need to include `//` on each line.

```sway
// hello world
```

```sway
// let's make a couple of lines
// commented.
```

You can also place comments at the ends of lines containing code.

```sway
fn main() {
    let baz = 8; // Eight is a lucky number
}
```

You can also do block comments

```sway
fn main() {
    /*
    You can write on multiple lines
    like this if you want
    */
    let baz = 8;
}
```
