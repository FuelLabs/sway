
## EXAMPLE:


```sh
forc template --url https://github.com/owner/template/ --project_name my_example_project
```

The command above fetches the HEAD of the `template` repo and searchs for `Forc.toml` at the root of the fetched repo. It will fetch the repo and preapare a new `Forc.toml` with new project name. Outputs everthing to `current_dir/project_name`.


```sh
forc template --url https://github.com/FuelLabs/sway/ --template_name counter --project_name my_example_project
```

The command above fetches the HEAD of the `sway` repo and searchs for `counter` example inside it (There is an example called `counter` under `sway/examples`). It will fetch the `counter` example and preapare a new `Forc.toml` with new project name. Outputs everthing to `current_dir/project_name`..
