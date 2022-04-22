# forc-parse-bytecode
Parse bytecode file into a debug format


## USAGE:
forc parse-bytecode <FILE_PATH>


## ARGS:

<_FILE_PATH_>


## OPTIONS:

`-h`, `--help` 

Print help information

## EXAMPLES:

Example with the initial project created using `forc init`:

```console
$ forc build -o obj
Compiled script "my-fuel-project".
Bytecode size is 28 bytes.
```

```console
my-second-project$ forc parse-bytecode obj

 half-word  byte  op               raw                notes
         0  0     JI(4)            [144, 0, 0, 4]     conditionally jumps to byte 16
         1  4     NOOP             [71, 0, 0, 0]
         2  8     Undefined        [0, 0, 0, 0]       data section offset lo (0)
         3  12    Undefined        [0, 0, 0, 28]      data section offset hi (28)
         4  16    LW(46, 12, 1)    [93, 184, 192, 1]
         5  20    ADD(46, 46, 12)  [16, 186, 227, 0]
         6  24    RET(0)           [36, 0, 0, 0]
```