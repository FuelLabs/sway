`bisect-forc.sh` will automatically run `forc bisect` searching for a different behaviour between two commits.

The script accepts three arguments:

```
bisect-forc.sh sway_project test 30s
```
1 - First argument is the sway project that will be compiled over and over until a different behavior is found;
2 - The second argument is which forc subcommand will be used. It defaults to `build`.

So, `forc` will be run as:

```
> forc <SECONDARGUMENT> --path <FIRSTARGUMENT>
```

3 - The third argument is a sleep that is taken between compilations to avoid notebooks enter into thermal throttle mode, or that weaker machines become unusable. Default to zero.
