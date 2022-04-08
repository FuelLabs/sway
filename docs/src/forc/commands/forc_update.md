# forc-update
Update dependencies in the Forc dependencies directory


## USAGE:
forc update [OPTIONS]


## OPTIONS:

`-c`, `--check` 

Checks if the dependencies have newer versions. Won't actually
perform the update, will output which ones are up-to-date and
outdated

`-d` <_TARGET_DEPENDENCY_>

Dependency to be updated. If not set, all dependencies will be
updated

`-h`, `--help` 

Print help information

`-p`, `--path` <_PATH_>

Path to the project, if not specified, current working directory
will be used