# Experimental Features

Sway compiler supports experimental features. Experimental features are used to:

- develop larger language features that might be unstable during the development (e.g., [References](https://github.com/FuelLabs/sway/issues/5063)),
- bring breaking changes in a controlled manner (e.g., [Partial equivalence](https://github.com/FuelLabs/sway/issues/6883)),
- ensure previous compiler behavior in case of incompatible changes ([e.g., New hashing](https://github.com/FuelLabs/sway/issues/7256)).

The list of [currently active](https://github.com/FuelLabs/sway/issues/?q=is%3Aissue%20state%3Aopen%20label%3Atracking-issue) and [already integrated](https://github.com/FuelLabs/sway/issues/?q=is%3Aissue%20state%3Aclosed%20label%3Atracking-issue) experimental features can be seen at Sway GitHub repository, as [issues marked with `tracking-issue` label](https://github.com/FuelLabs/sway/issues/?q=is%3Aissue%20label%3Atracking-issue). Each tracking issue contains detailed description of its experimental feature, as well as any breaking changes that the feature brings.

## Enabling and Disabling Experimental Features

Each experimental feature has a unique _feature flag_ defined for it. Feature flags are used to early opt-in for a feature, or to opt-out if the feature is already enabled by default, and you want to have the previous compiler behavior.

E.g., a feature flag for the [Const Generics](https://github.com/FuelLabs/sway/issues/6860) feature is `const_generics`.

Experimental features can be enabled and disabled using the `Forc.toml`, `forc` CLI, or environment variables.

If some feature is turned on in the `Forc.toml`, it can be turned off by the CLI or by environment variables.

If a feature is _not_ turned on in the `Forc.toml`, it can still be turned on by the CLI and environment variables.

**Environment variables overwrite CLI arguments, which overwrite the `Forc.toml` configuration.**

### `Forc.toml`

To enable and disable experimental features for a package, use the `experimental` field inside of the `Forc.toml`'s `[project]` section. Each experimental feature can be turned on or off, by setting its feature flag to `true` or `false`, respectively. If a feature flag of some existing experimental feature is not listed in the `experimental` field, the default value for enabling that feature will be used.

```toml
[project]
... # Other project fields.
experimental = { some_feature = true, some_other_feature = false }
```

### `forc` CLI

In `forc` CLI, opting in and out of an experimental feature is done by using two compiler flags, `--experimental` and `--no-experimental`, respectively:

```console
forc build --experimental some_feature --no-experimental some_other_feature
```

To opt-in or out of several experimental features, separate them by comma:

```console
forc build --experimental some_feature_1,some_feature_2 --no-experimental some_other_feature_1,some_other_feature_2
```

### Environment Variables

To enable and disable experimental features on the environment level, use the environment variables `FORC_EXPERIMENTAL` and `FORC_NO_EXPERIMENTAL`, respectively. Here are some examples that set those environment variables prior running `forc`:

```console
FORC_EXPERIMENTAL=some_feature,other_feature forc build
FORC_NO_EXPERIMENTAL=some_feature,other_feature forc build
FORC_EXPERIMENTAL=some_feature FORC_NO_EXPERIMENTAL=other_feature forc build
```

## Conditional Compilation

Experimental features are supported in [conditional compilation](../reference/attributes.md#cfg) using the `#[cfg]` attribute. For each `<feature_flag>` there is a boolean argument named `experimental_<feature_flag>` which can be set to `true` or `false`. The annotated code will be compiled only:

- if the feature is enabled during compilation and the `experimental_<feature_flag>` is set to `true`.
- if the feature is _not_ enabled during compilation and the `experimental_<feature_flag>` is set to `false`.

If the conditional compilation depends upon several experimental features, multiple `#[cfg]` attributes can be combined. E.g.:

```sway
#[cfg(experimental_some_feature = true)]
fn conditionally_compiled() {
    log("This is compiled only if `some_feature` is enabled.");
}

#[cfg(experimental_some_feature = false)]
fn conditionally_compiled() {
    log("This is compiled only if `some_feature` is disabled.");
}

#[cfg(experimental_some_feature = true)]
#[cfg(experimental_some_other_feature = true)]
fn conditionally_compiled() {
    log("This is compiled only if both `some_feature` and `some_other_feature` are enabled.");
}
```

## Tracking Experimental Features

- `abi_type_aliases` — keeps the JSON ABI emitter from expanding type aliases into their target type when serializing a contract's ABI, allowing the published JSON to preserve the original alias names.

See the tracking issue for this feature [here](https://github.com/FuelLabs/sway/issues/7486).
