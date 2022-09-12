<!-- markdownlint-disable MD041 -->

## EXAMPLE

```sh
forc template --url https://github.com/owner/template/ --project_name my_example_project
```

The command above fetches the `HEAD` of the `template` repo and searches for `Forc.toml` at the root of the fetched repo. It will fetch the repo and prepare a new `Forc.toml` with the new project name. Outputs everything to `current_dir/project_name`.

```sh
forc template --url https://github.com/FuelLabs/sway --template_name counter --project_name my_example_project
```

The command above fetches the HEAD of the `sway` repo and searches for `counter` example inside it (there is an example called `counter` under `sway/examples`). It will fetch the `counter` example and prepare a new `Forc.toml` with the new project name. Outputs everything to `current_dir/project_name`.
