<!-- markdownlint-disable MD041 -->

# Migrating Sway projects

`forc-migrate` guides you through breaking changes between Sway versions. It fully or semiautomatically adapts your code, making it compatible with the next breaking change version of Sway.

`forc-migrate` migrates the code to the _next_ breaking change version of Sway. That means, if you want to migrate to, e.g., Sway v0.**67**.0, you will need to use the _latest v0.**66**.x_ version of the `forc-migrate`.

For example, let's say that your Sway project is on version _v0.66.1_, and that the latest v0.66 version is _v0.66.42_. You should first update your Fuel toolchain to version _v0.66.42_ of `forc`, and compile your project with that version:

```text
fuelup component add forc@0.66.42
```

Sway guarantees that all the versions with the same minor version, _0.66_ in the above example, are compatible. That means that the latest patch version, _0.66.42_ in the example, will correctly compile your project.

## Showing the breaking changes

Once you've installed the latest non-breaking version of `forc-migrate`, use the `show` command to make yourself familiar with the upcoming breaking changes:

```text
forc migrate show
```

A typical output of the `show` command will look like this:

```text
Breaking change features:
  - storage_domains    (https://github.com/FuelLabs/sway/issues/6701)
  - references         (https://github.com/FuelLabs/sway/issues/5063)

Migration steps (1 manual and 1 semiautomatic):
storage_domains
  [M] Review explicitly defined slot keys in storage declarations (`in` keywords)

references
  [S] Replace `ref mut` function parameters with `&mut`

Experimental feature flags:
- for Forc.toml:  experimental = { storage_domains = true, references = true }
- for CLI:        --experimental storage_domains,references
```

The output will contain:

- the upcoming breaking change features, `storage_domains` and `references` in this example,
- their tracking issues on GitHub, with detailed migration guides,
- and the migration steps potentially required to migrate existing code.

The migration steps can be _manual_, _semiautomatic_, or fully _automatic_. They are marked in the output with `[M]`, `[S]`, and `[A]`, respectively.

The `show` command will also provide experimental feature flags that will be needed during the migration, as explained in the next chapter.

## Migrating a single Sway project

Let's assume that we want to migrate a Sway project called `my_project` that depends on `std` and a `third_party_lib`.

First, we will go to the folder that contains `my_project`, e.g.: `cd my_project`. All of the upcoming CLI commands assume that we are running the `forc-migrate` tool within the `my_project` folder.

Before migrating the code, make sure that the project builds without any errors by running:

```text
forc build
```

### Check the migration summary

Next, let's `check` the project first. The `check` command will dry-run the migration steps. It will not do any changes in code, but will provide a detailed information of all the places in code that need to be either reviewed or changed during the migration process. The `check` command will also provide a rough time estimate for the migration.

```text
forc migrate check
```

The output of the `check` command will end in a summary of the migration effort, containing:

- the number of occurrences of a particular migration step in the project's code,
- the rough migration effort estimate for each migration step,
- and the rough total migration effort.

```text
Migration effort:

storage_domains
  [M] Review explicitly defined slot keys in storage declarations (`in` keywords)
      Occurrences:     3    Migration effort (hh::mm): ~00:06

references
  [S] Replace `ref mut` function parameters with `&mut`
      Occurrences:    18    Migration effort (hh::mm): ~01:30

Total migration effort (hh::mm): ~01:36
```

Before the summary, instructions will be shown for each migration step. A typical instruction output for a single migration step will contain:

- the name of the step,
- the places in code affected by the migration step,
- and the short help with a link to the detailed migration guide.

```text
info: [references] Replace `ref mut` function parameters with `&mut`
  --> my_project/src/main.sw:30:51
   |
...
30 | fn ref_mut_fn(ref mut x: u64) {}
   |               ---------
...
35 | fn another_ref_mut_fn(ref mut arg: S) {}
   |                       -----------
   |
   = help: Migration will replace `ref mut` function parameters with `&mut`.
   = help: E.g., `ref mut x: u64` will become `x: &mut u64`.
   = help:  
   = help: After the migration, you will still need to:
   = help: - change function callers, by adding `&mut` to passed parameters.
   = help: - change function bodies, by dereferencing (`*`) parameters where needed.
   = help:  
   = help: For a detailed migration guide see: https://github.com/FuelLabs/sway/issues/5063
```

### Update dependencies

Before running the migrations on the project itself, **first update the project dependencies to the versions that use the next Sway breaking change version**.

In our example, the `my_project`'s `Forc.toml` file will have the `[dependencies]` section similar to this one:

```toml
[dependencies]
std = { git = "https://github.com/FuelLabs/sway", tag = "v0.66.1" }
third_party_lib = { git = "https://github.com/ThirdParty/swaylib", tag = "v1.0.0" }
```

Assuming that the `third_party_lib` version compatible with Sway v0.67.0 is the version v2.0.0 we will end up in the following changes:

```toml
[dependencies]
# Changed v0.66.1 -> v0.67.0
std = { git = "https://github.com/FuelLabs/sway", tag = "v0.67.0" }
# Changed v1.0.0  -> v2.0.0
third_party_lib = { git = "https://github.com/ThirdParty/swaylib", tag = "v2.0.0" }
```

Run `forc build` to make sure that the project still compiles. **At this point, it is very likely that you will need to compile the project with the experimental features turned on.** The reason is the likelihood that either the new `std` or the `third_party_lib` uses the new Sway features.

To compile the project with experimental features, you can take the feature flags from the `forc migrate show` output, and place them either in the `[build-profile]` section of the projects `Forc.toml` file, or pass them to `forc build` via the command line.

```text
Experimental feature flags:
- for Forc.toml:  experimental = { storage_domains = true, references = true }
- for CLI:        --experimental storage_domains,references
```

In the remaining part of this tutorial, we will be passing the feature flags via the command line. E.g.:

```text
forc build --experimental storage_domains,references
```

### Run the migrations

Once the `my_project` successfully builds with updated dependencies, we can `run` the migration steps on it. E.g.:

```text
forc migrate run --experimental storage_domains,references
```

The `run` command will execute the migration steps, and guide you through the migration process. For each migration step, the output of the step can be one of the following:

| Step output | Meaning |
| ----------- | ------- |
| Checked     | The step is executed and does not require any changes in code. No action needed. |
| Review      | The step suggests a manual code review. |
| Changing    | The step is automatically changing the code. There might be additional manual actions needed. |

At the end of the `run`, the migration will either guide you to:

- `Continue` the migration process by performing the manual actions and re-running the `forc migrate run` afterwards,
- or will mark the migration process as `Finished`. At this point, your project will be compatible with the next breaking change version of Sway.

`forc migrate`, same like `forc fmt`, does its best to preserve the positions of comments in the modified code. This is a challenging task, especially if migration steps remove parts of the code. **It is a good practice to always `diff` the changes done within migration steps and check if the comments are placed where expected.**

## Migrating workspaces

To migrate a workspace, you will need to migrate each workspace member separately, following the above procedure. The projects should be migrated in order of their dependencies.

> **Note**: There is a know limitation when running `forc migrate` on projects that are listed as workspace members. `forc migrate` will run, but possibly not find all the occurrences in code that need to be migrated. Therefore, **before running migrations on projects that are workspace members, remove them temporarily from the list of workspace `members`**.

## Additional after-migration steps

There are some additional manual steps that might be needed after the migration.

E.g., if tests use hardcoded contract IDs, those need to be changed, because the new version of Sway will, very likely, produce different bytecode.
